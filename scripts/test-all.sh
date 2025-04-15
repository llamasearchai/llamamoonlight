#!/bin/bash

# --- Master Testing Script for Llama Ecosystem ---

# ANSI color codes
GREEN="\033[0;32m"
YELLOW="\033[0;33m"
RED="\033[0;31m"
BLUE="\033[0;34m"
MAGENTA="\033[0;35m"
CYAN="\033[0;36m"
NC="\033[0m" # No Color
BOLD="\033[1m"

# --- Helper Functions for Colored Output ---
green_echo() {
    echo -e "${GREEN}$1${NC}"
}

yellow_echo() {
    echo -e "${YELLOW}$1${NC}"
}

red_echo() {
    echo -e "${RED}$1${NC}"
}

blue_echo() {
    echo -e "${BLUE}$1${NC}"
}

magenta_echo() {
    echo -e "${MAGENTA}$1${NC}"
}

cyan_echo() {
    echo -e "${CYAN}$1${NC}"
}

bold_echo() {
    echo -e "${BOLD}$1${NC}"
}

# Print a banner
print_banner() {
    magenta_echo ""
    magenta_echo "  _      _                          ______                       _                 "
    magenta_echo " | |    | |                        |  ____|                     | |                "
    magenta_echo " | |    | | __ _ _ __ ___   __ _   | |__   ___ ___  ___ _   _ __| |_ ___ _ __ ___  "
    magenta_echo " | |    | |/ _\` | '_ \` _ \ / _\` |  |  __| / __/ _ \/ __| | | / _\` __/ _ \ '_ \` _ \ "
    magenta_echo " | |____| | (_| | | | | | | (_| |  | |___| (_| (_) \__ \ |_| | (_| |_  __/ | | | | |"
    magenta_echo " |______|_|\__,_|_| |_| |_|\__,_|  |______\___\___/|___/\__, |\__\__\___|_| |_| |_|"
    magenta_echo "                                                          __/ |                    "
    magenta_echo "                                                         |___/                     "
    magenta_echo ""
    yellow_echo "                     Complete Test Suite for the Llama Ecosystem"
    echo ""
}

# Print section header
print_section() {
    echo ""
    bold_echo "${BLUE}======== $1 ========${NC}"
    echo ""
}

# Run a command with a progress message
run_command() {
    local cmd="$1"
    local success_msg="$2"
    local error_msg="$3"
    
    echo -n "⏳ "
    yellow_echo "Running: $cmd"
    eval "$cmd"
    
    if [ $? -eq 0 ]; then
        echo -n "✅ "
        green_echo "$success_msg"
        return 0
    else
        echo -n "❌ "
        red_echo "$error_msg"
        return 1
    fi
}

# Test a crate
test_crate() {
    local crate_name="$1"
    local crate_dir="$2"
    
    if [ ! -d "$crate_dir" ]; then
        yellow_echo "⚠️ Skipping $crate_name: Directory not found"
        return 0
    fi
    
    print_section "Testing $crate_name"
    
    # Navigate to the crate directory
    cd "$crate_dir" || {
        red_echo "Error: Cannot navigate to $crate_dir"
        return 1
    }
    
    # Run tests
    cyan_echo "Running tests for $crate_name"
    if ! run_command "cargo test" "$crate_name tests passed" "$crate_name tests failed"; then
        cd - > /dev/null
        return 1
    fi
    
    # Run clippy
    cyan_echo "Running clippy for $crate_name"
    if ! run_command "cargo clippy -- -D warnings" "$crate_name clippy checks passed" "$crate_name clippy checks failed"; then
        cd - > /dev/null
        return 1
    fi
    
    # Format code
    cyan_echo "Formatting code for $crate_name"
    if ! run_command "cargo fmt --all -- --check" "$crate_name code is properly formatted" "$crate_name code formatting issues found"; then
        cd - > /dev/null
        return 1
    fi
    
    # Build docs
    cyan_echo "Building documentation for $crate_name"
    if ! run_command "cargo doc --no-deps" "$crate_name documentation generated" "$crate_name documentation generation failed"; then
        cd - > /dev/null
        return 1
    fi
    
    # Check for examples
    if [ -d "examples" ]; then
        cyan_echo "Running examples for $crate_name"
        local examples=$(find examples -name "*.rs" -exec basename {} .rs \;)
        
        if [ -z "$examples" ]; then
            yellow_echo "No examples found for $crate_name, skipping."
        else
            for example in $examples; do
                if ! run_command "cargo run --example $example" "Example $example ran successfully" "Example $example failed"; then
                    cd - > /dev/null
                    return 1
                fi
            done
        fi
    fi
    
    # Check for benchmarks
    if [ -d "benches" ]; then
        cyan_echo "Running benchmarks for $crate_name"
        if ! run_command "cargo bench" "Benchmarks completed" "Benchmarks failed"; then
            cd - > /dev/null
            return 1
        fi
    fi
    
    # Verify package
    cyan_echo "Verifying package for $crate_name"
    if ! run_command "cargo package --allow-dirty" "Package verification successful" "Package verification failed"; then
        cd - > /dev/null
        return 1
    fi
    
    # Return to the original directory
    cd - > /dev/null
    
    green_echo "✅ $crate_name passed all tests!"
    return 0
}

# Generate a crate status report
generate_report() {
    local crates=("$@")
    local pass_count=0
    local fail_count=0
    local skip_count=0
    
    print_section "Crate Status Report"
    
    for crate in "${crates[@]}"; do
        if [ -d "$crate" ]; then
            if grep -q "PASS" <<< $(echo "$CRATE_RESULTS" | grep "^$crate:"); then
                echo -e "${GREEN}✅ $crate: PASS${NC}"
                ((pass_count++))
            else
                echo -e "${RED}❌ $crate: FAIL${NC}"
                ((fail_count++))
            fi
        else
            echo -e "${YELLOW}⚠️ $crate: SKIP (not found)${NC}"
            ((skip_count++))
        fi
    done
    
    echo ""
    green_echo "Crates Passed: $pass_count"
    
    if [ "$fail_count" -gt 0 ]; then
        red_echo "Crates Failed: $fail_count"
    else
        green_echo "Crates Failed: 0"
    fi
    
    yellow_echo "Crates Skipped: $skip_count"
    echo ""
    
    if [ "$fail_count" -eq 0 ]; then
        green_echo "\n🎉 All available crates passed the tests! The ecosystem is in good health."
    else
        red_echo "\n⚠️ Some crates failed the tests. Please fix the issues before publishing."
    fi
}

# Main function
main() {
    print_banner
    
    # Record script start time
    local start_time=$(date +%s)
    
    # Check if we're in the right directory
    if [ ! -d "llama-headers-rs" ] && [ ! -d "llama-moonlight" ]; then
        red_echo "Error: Cannot find required directories. Please run this script from the ecosystem root directory."
        exit 1
    fi
    
    # Check for required tools
    print_section "Checking Required Tools"
    
    local tools=("cargo" "rustc" "rustup")
    local missing=()
    
    for tool in "${tools[@]}"; do
        if ! command -v "$tool" > /dev/null; then
            missing+=("$tool")
        fi
    done
    
    if [ ${#missing[@]} -gt 0 ]; then
        red_echo "Missing required tools: ${missing[*]}"
        red_echo "Please install them before proceeding."
        exit 1
    fi
    
    green_echo "All required tools are installed."
    
    # Define our crates
    declare -a crates=(
        "llama-headers-rs"
        "llama-moonlight"
        # Add more crates here as they are developed
        # "llama-cloudflare"
        # "llama-mlx"
        # "llama-agents"
    )
    
    # Variable to store crate test results
    CRATE_RESULTS=""
    
    # Initialize counters
    local pass_count=0
    local fail_count=0
    
    # Test each crate
    for crate in "${crates[@]}"; do
        if test_crate "$crate" "$crate"; then
            CRATE_RESULTS+="$crate: PASS\n"
            ((pass_count++))
        else
            CRATE_RESULTS+="$crate: FAIL\n"
            ((fail_count++))
        fi
    done
    
    # Record script end time
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    # Generate report
    generate_report "${crates[@]}"
    
    # Print summary
    print_section "Test Summary"
    cyan_echo "Total Duration: $duration seconds"
    
    # Create badges for GitHub
    print_section "GitHub Badges"
    
    for crate in "${crates[@]}"; do
        if [ -d "$crate" ]; then
            if grep -q "PASS" <<< $(echo "$CRATE_RESULTS" | grep "^$crate:"); then
                echo -e "[![$crate](https://img.shields.io/badge/$crate-passing-brightgreen.svg)](https://github.com/yourusername/llama-ecosystem/tree/main/$crate)"
            else
                echo -e "[![$crate](https://img.shields.io/badge/$crate-failing-red.svg)](https://github.com/yourusername/llama-ecosystem/tree/main/$crate)"
            fi
        fi
    done
    
    # Provide next steps
    print_section "Next Steps"
    
    if [ "$fail_count" -eq 0 ]; then
        cyan_echo "All crates are ready for publishing! Run the following commands to publish each crate:"
        echo ""
        
        for crate in "${crates[@]}"; do
            if [ -d "$crate" ]; then
                yellow_echo "cd $crate && cargo publish && cd .."
            fi
        done
    else
        red_echo "Please fix the failing crates before publishing."
    fi
}

# Run main function
main 