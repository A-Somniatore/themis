#!/bin/bash
# Themis GitHub Action Entrypoint
# This script processes inputs and runs the appropriate Themis command

set -e

# Input arguments (passed from action.yml)
COMMAND="${1}"
CONTRACT="${2}"
OLD_CONTRACT="${3}"
NEW_CONTRACT="${4}"
FORMAT="${5}"
LANGUAGE="${6}"
OUTPUT="${7}"
CONFIG="${8}"
FAIL_ON_WARNINGS="${9}"
RULES="${10}"
ALLOW_BREAKING="${11}"
WORKING_DIR="${12}"
VERSION="${13}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Change to working directory if specified
if [ -n "$WORKING_DIR" ] && [ "$WORKING_DIR" != "." ]; then
    log_info "Changing to working directory: $WORKING_DIR"
    cd "$WORKING_DIR"
fi

# Verify Themis is available
if ! command -v themis &> /dev/null; then
    log_error "Themis CLI not found!"
    exit 1
fi

log_info "Using Themis version: $(themis --version 2>/dev/null || echo 'unknown')"

# Build the command based on input
build_command() {
    local cmd="themis"
    
    case "$COMMAND" in
        validate)
            cmd="$cmd validate"
            if [ -n "$CONTRACT" ]; then
                cmd="$cmd \"$CONTRACT\""
            else
                log_error "Contract path is required for validate command"
                exit 1
            fi
            if [ -n "$FORMAT" ]; then
                cmd="$cmd --format $FORMAT"
            fi
            ;;
            
        lint)
            cmd="$cmd lint"
            if [ -n "$CONTRACT" ]; then
                cmd="$cmd \"$CONTRACT\""
            else
                log_error "Contract path is required for lint command"
                exit 1
            fi
            if [ -n "$CONFIG" ]; then
                cmd="$cmd --config \"$CONFIG\""
            fi
            if [ "$RULES" != "all" ] && [ -n "$RULES" ]; then
                cmd="$cmd --rules $RULES"
            fi
            ;;
            
        compat)
            cmd="$cmd compat"
            if [ -n "$OLD_CONTRACT" ] && [ -n "$NEW_CONTRACT" ]; then
                cmd="$cmd \"$OLD_CONTRACT\" \"$NEW_CONTRACT\""
            elif [ -n "$CONTRACT" ]; then
                # Single contract against previous version (needs git)
                cmd="$cmd \"$CONTRACT\""
            else
                log_error "Contract paths required for compat command"
                exit 1
            fi
            if [ "$ALLOW_BREAKING" = "true" ]; then
                cmd="$cmd --allow-breaking"
            fi
            ;;
            
        codegen)
            cmd="$cmd codegen"
            if [ -n "$CONTRACT" ]; then
                cmd="$cmd \"$CONTRACT\""
            else
                log_error "Contract path is required for codegen command"
                exit 1
            fi
            if [ -n "$LANGUAGE" ]; then
                cmd="$cmd --language $LANGUAGE"
            else
                log_error "Language is required for codegen command"
                exit 1
            fi
            if [ -n "$OUTPUT" ]; then
                cmd="$cmd --output \"$OUTPUT\""
            fi
            ;;
            
        *)
            log_error "Unknown command: $COMMAND"
            log_info "Valid commands: validate, lint, compat, codegen"
            exit 1
            ;;
    esac
    
    echo "$cmd"
}

# Execute the command
run_themis() {
    local cmd
    cmd=$(build_command)
    
    log_info "Running: $cmd"
    echo "---"
    
    # Create a temp file for output
    local output_file
    output_file=$(mktemp)
    local exit_code=0
    
    # Run the command and capture output
    eval "$cmd" 2>&1 | tee "$output_file" || exit_code=$?
    
    echo "---"
    
    # Parse output for GitHub outputs
    local issues_count=0
    local breaking_count=0
    
    # Count issues (look for patterns in output)
    if [ -f "$output_file" ]; then
        issues_count=$(grep -c -E "(error|warning|issue)" "$output_file" 2>/dev/null || echo "0")
        breaking_count=$(grep -c -i "breaking" "$output_file" 2>/dev/null || echo "0")
    fi
    
    # Set GitHub outputs
    if [ -n "$GITHUB_OUTPUT" ]; then
        echo "result=$([ $exit_code -eq 0 ] && echo 'success' || echo 'failure')" >> "$GITHUB_OUTPUT"
        echo "issues-count=$issues_count" >> "$GITHUB_OUTPUT"
        echo "breaking-changes=$breaking_count" >> "$GITHUB_OUTPUT"
    fi
    
    # Clean up
    rm -f "$output_file"
    
    # Handle exit code
    if [ $exit_code -ne 0 ]; then
        log_error "Themis command failed with exit code $exit_code"
        return $exit_code
    fi
    
    # Check for warnings if fail-on-warnings is set
    if [ "$FAIL_ON_WARNINGS" = "true" ] && [ "$issues_count" -gt 0 ]; then
        log_warning "Failing due to warnings (fail-on-warnings is enabled)"
        return 1
    fi
    
    log_success "Themis command completed successfully"
    return 0
}

# Main execution
main() {
    log_info "Starting Themis Contract Governance Action"
    log_info "Command: $COMMAND"
    
    run_themis
    local result=$?
    
    if [ $result -eq 0 ]; then
        log_success "Action completed successfully"
    else
        log_error "Action failed"
    fi
    
    exit $result
}

main
