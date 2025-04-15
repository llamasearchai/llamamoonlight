#!/bin/bash

# --- Test, Benchmark, and Publish for llama-moonlight ---

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
    magenta_echo "  _      _                          __  __                     _ _       _     _   "
    magenta_echo " | |    | |                        |  \/  |                   | (_)     | |   | |  "
    magenta_echo " | |    | | __ _ _ __ ___   __ _   | \  / | ___   ___  _ __  | |_  __ _| |__ | |_ "
    magenta_echo " | |    | |/ _\` | '_ \` _ \ / _\` |  | |\/| |/ _ \ / _ \| '_ \ | | |/ _\` | '_ \| __|"
    magenta_echo " | |____| | (_| | | | | | | (_| |  | |  | | (_) | (_) | | | || | | (_| | | | | |_ "
    magenta_echo " |______|_|\__,_|_| |_| |_|\__,_|  |_|  |_|\___/ \___/|_| |_||_|_|\__, |_| |_|\__|"
    magenta_echo "                                                                    __/ |          "
    magenta_echo "                                                                   |___/           "
    magenta_echo ""
    yellow_echo "                A powerful browser automation framework for Rust"
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

# Test a specific crate
test_crate() {
    local crate_dir="$1"
    local crate_name="$2"
    
    print_section "Testing $crate_name"
    
    cd "$crate_dir" || {
        red_echo "Error: Cannot navigate to $crate_dir"
        return 1
    }
    
    # Run all tests
    run_command "cargo test" "$crate_name tests passed" "$crate_name tests failed" || return 1
    
    # Run clippy
    run_command "cargo clippy -- -D warnings" "$crate_name clippy checks passed" "$crate_name clippy checks failed" || return 1
    
    # Check formatting
    run_command "cargo fmt --all -- --check" "$crate_name code formatting is correct" "$crate_name code formatting issues found" || return 1
    
    # Generate docs
    run_command "cargo doc --no-deps" "$crate_name documentation generated" "$crate_name documentation generation failed" || return 1
    
    cd - > /dev/null
    return 0
}

# Main function
main() {
    print_banner
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        red_echo "Error: Cannot find Cargo.toml. Please run this script from the llama-moonlight directory."
        exit 1
    fi
    
    # Check for all required tools
    print_section "Checking prerequisites"
    
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
    
    # Check for crate workspace components
    print_section "Checking workspace structure"
    
    local components=("llama-moonlight-core" "llama-moonlight-cli" "llama-moonlight-pool" "llama-moonlight-rxt" "llama-moonlight-testutil" "llama-moonlight-mlx")
    local missing=()
    
    for component in "${components[@]}"; do
        if [ ! -d "$component" ]; then
            missing+=("$component")
        fi
    done
    
    if [ ${#missing[@]} -gt 0 ]; then
        yellow_echo "Missing workspace components: ${missing[*]}"
        yellow_echo "Some tests will be skipped."
    else
        green_echo "All workspace components are present."
    fi
    
    # First build all crates
    print_section "Building all crates"
    run_command "cargo build --all" "All crates built successfully" "Failed to build some crates" || exit 1
    
    # Test main crate
    test_crate "." "llama-moonlight" || exit 1
    
    # Test each workspace component
    for component in "${components[@]}"; do
        if [ -d "$component" ]; then
            test_crate "$component" "$component" || exit 1
        fi
    done
    
    # Run benchmarks if they exist
    print_section "Running benchmarks"
    if [ -d "benches" ]; then
        run_command "cargo bench" "Benchmarks completed successfully" "Benchmarks failed"
    else
        yellow_echo "No benchmarks found, skipping."
    fi
    
    # Run examples
    print_section "Testing examples"
    if [ -d "examples" ]; then
        local examples=$(find examples -name "*.rs" -exec basename {} .rs \;)
        
        if [ -z "$examples" ]; then
            yellow_echo "No examples found, skipping."
        else
            for example in $examples; do
                run_command "cargo run --example $example" "Example $example ran successfully" "Example $example failed"
            done
        fi
    else
        yellow_echo "No examples directory found, skipping."
    fi
    
    # Create a package to verify it will publish correctly
    print_section "Verifying package for publishing"
    run_command "cargo package --allow-dirty" "Package verification successful" "Package verification failed" || exit 1
    
    # Show instructions for publishing
    print_section "Publishing instructions"
    cyan_echo "Your crate is ready to publish! Run the following command to publish:"
    yellow_echo "cargo publish"
    
    green_echo "\nCongratulations! 🎉 llama-moonlight has been tested and is ready to be published!"
}

# Run the main function
main 