#!/bin/bash

# --- Test, Benchmark, and Publish for llama-headers-rs ---

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
    magenta_echo "  _      _                                _    _                _               "
    magenta_echo " | |    | |                              | |  | |              | |              "
    magenta_echo " | |    | | __ _ _ __ ___   __ _       __| |__| | ___  __ _  __| | ___ _ __ ___ "
    magenta_echo " | |    | |/ _\` | '_ \` _ \\ / _\` |     / _\` |__  |/ _ \\/ _\` |/ _\` |/ _ \\ '__/ __|"
    magenta_echo " | |____| | (_| | | | | | | (_| |    | (_| |  | |  __/ (_| | (_| |  __/ |  \\__ \\"
    magenta_echo " |______|_|\\__,_|_| |_| |_|\\__,_|     \\__,_|  |_|\\___|\\__,_|\\__,_|\\___|_|  |___/"
    magenta_echo ""
    yellow_echo "                A sophisticated HTTP header generator for Rust"
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
    else
        echo -n "❌ "
        red_echo "$error_msg"
        exit 1
    fi
}

# Main function
main() {
    print_banner
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        red_echo "Error: Cannot find Cargo.toml. Please run this script from the llama-headers-rs directory."
        exit 1
    fi
    
    # Verify Cargo.toml formatting
    print_section "Validating Cargo.toml"
    run_command "cargo check --quiet" "Cargo.toml is valid." "Cargo.toml has errors."
    
    # Run unit tests
    print_section "Running Unit Tests"
    run_command "cargo test" "All tests passed successfully." "Some tests failed."
    
    # Run clippy for linting
    print_section "Running Clippy Linter"
    run_command "cargo clippy -- -D warnings" "Code passed linting." "Linting found issues."
    
    # Format code
    print_section "Formatting Code"
    run_command "cargo fmt --all" "Code formatted successfully." "Code formatting failed."
    
    # Run benchmarks
    print_section "Running Benchmarks"
    run_command "cargo bench" "Benchmarks completed." "Benchmarks failed."
    
    # Run examples
    print_section "Testing Examples"
    run_command "cargo run --example simple" "Simple example runs successfully." "Simple example failed."
    
    # Check documentation
    print_section "Checking Documentation"
    run_command "cargo doc --no-deps" "Documentation generated successfully." "Documentation generation failed."
    
    # Prepare package verification 
    print_section "Verifying Package"
    run_command "cargo package --allow-dirty" "Package verification successful." "Package verification failed."
    
    # Show publish command but don't execute
    print_section "Publishing Instructions"
    cyan_echo "Your crate is ready to publish! Run the following command to publish:"
    yellow_echo "cargo publish"
    
    green_echo "\nCongratulations! 🎉 llama-headers-rs has been tested and is ready for publishing!"
}

# Run main function
main 