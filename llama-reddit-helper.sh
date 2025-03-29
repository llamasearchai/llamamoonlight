#!/usr/bin/env bash
# ===============================================================================
# Llama-Reddit Helper Script
# ===============================================================================
# This script provides shortcuts for common Llama-Reddit operations
#
# Usage:
#   ./llama-reddit-helper.sh <command> [options]
#
# Commands:
#   search <subreddit> <keyword> [limit]      - Search for posts with keyword
#   top <subreddit> [limit]                   - Get top posts from subreddit
#   hot <subreddit> [limit]                   - Get hot posts from subreddit
#   images <subreddit> [limit]                - Download images from subreddit
#   csv <subreddit> [keyword] [limit]         - Export data to CSV
#   help                                      - Show this help message
#
# Examples:
#   ./llama-reddit-helper.sh search programming rust 50
#   ./llama-reddit-helper.sh top wallpapers 100
#   ./llama-reddit-helper.sh images art 25
#   ./llama-reddit-helper.sh csv science "quantum computing" 75
# ===============================================================================

# ANSI color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Path to the Llama-Reddit executable
LLAMA_REDDIT="./target/release/llama-reddit"

# Check if the executable exists
if [ ! -f "$LLAMA_REDDIT" ]; then
    echo -e "${RED}Error: Llama-Reddit executable not found at $LLAMA_REDDIT${NC}"
    echo -e "${YELLOW}Make sure you've built the project with 'cargo build --release'${NC}"
    exit 1
fi

# Show help message
show_help() {
    echo -e "${BLUE}Llama-Reddit Helper Script${NC}"
    echo
    echo "Usage:"
    echo "  ./llama-reddit-helper.sh <command> [options]"
    echo
    echo "Commands:"
    echo "  search <subreddit> <keyword> [limit]      - Search for posts with keyword"
    echo "  top <subreddit> [limit]                   - Get top posts from subreddit"
    echo "  hot <subreddit> [limit]                   - Get hot posts from subreddit" 
    echo "  images <subreddit> [limit]                - Download images from subreddit"
    echo "  csv <subreddit> [keyword] [limit]         - Export data to CSV"
    echo "  help                                      - Show this help message"
    echo
    echo "Examples:"
    echo "  ./llama-reddit-helper.sh search programming rust 50"
    echo "  ./llama-reddit-helper.sh top wallpapers 100"
    echo "  ./llama-reddit-helper.sh images art 25"
    echo "  ./llama-reddit-helper.sh csv science \"quantum computing\" 75"
    echo
}

# Handle commands
case "$1" in
    search)
        if [ -z "$2" ] || [ -z "$3" ]; then
            echo -e "${RED}Error: Missing required arguments${NC}"
            echo "Usage: ./llama-reddit-helper.sh search <subreddit> <keyword> [limit]"
            exit 1
        fi
        
        subreddit="$2"
        keyword="$3"
        limit="${4:-25}"  # Default to 25 if not provided
        
        echo -e "${GREEN}Searching r/$subreddit for \"$keyword\" (limit: $limit)...${NC}"
        $LLAMA_REDDIT -s "$subreddit" -k "$keyword" -l "$limit"
        ;;
        
    top)
        if [ -z "$2" ]; then
            echo -e "${RED}Error: Missing required argument${NC}"
            echo "Usage: ./llama-reddit-helper.sh top <subreddit> [limit]"
            exit 1
        fi
        
        subreddit="$2"
        limit="${3:-25}"  # Default to 25 if not provided
        
        echo -e "${GREEN}Getting top posts from r/$subreddit (limit: $limit)...${NC}"
        $LLAMA_REDDIT -s "$subreddit" --listing top -l "$limit"
        ;;
        
    hot)
        if [ -z "$2" ]; then
            echo -e "${RED}Error: Missing required argument${NC}"
            echo "Usage: ./llama-reddit-helper.sh hot <subreddit> [limit]"
            exit 1
        fi
        
        subreddit="$2"
        limit="${3:-25}"  # Default to 25 if not provided
        
        echo -e "${GREEN}Getting hot posts from r/$subreddit (limit: $limit)...${NC}"
        $LLAMA_REDDIT -s "$subreddit" --listing hot -l "$limit"
        ;;
        
    images)
        if [ -z "$2" ]; then
            echo -e "${RED}Error: Missing required argument${NC}"
            echo "Usage: ./llama-reddit-helper.sh images <subreddit> [limit]"
            exit 1
        fi
        
        subreddit="$2"
        limit="${3:-25}"  # Default to 25 if not provided
        
        echo -e "${GREEN}Downloading images from r/$subreddit (limit: $limit)...${NC}"
        $LLAMA_REDDIT -s "$subreddit" -l "$limit" --save-images --skip-duplicates
        ;;
        
    csv)
        if [ -z "$2" ]; then
            echo -e "${RED}Error: Missing required argument${NC}"
            echo "Usage: ./llama-reddit-helper.sh csv <subreddit> [keyword] [limit]"
            exit 1
        fi
        
        subreddit="$2"
        keyword="$3"  # Optional
        limit="${4:-25}"  # Default to 25 if not provided
        
        if [ -z "$keyword" ]; then
            echo -e "${GREEN}Exporting data from r/$subreddit to CSV (limit: $limit)...${NC}"
            $LLAMA_REDDIT -s "$subreddit" -l "$limit" -o csv
        else
            echo -e "${GREEN}Exporting data from r/$subreddit with keyword \"$keyword\" to CSV (limit: $limit)...${NC}"
            $LLAMA_REDDIT -s "$subreddit" -k "$keyword" -l "$limit" -o csv
        fi
        ;;
        
    help|*)
        show_help
        ;;
esac
