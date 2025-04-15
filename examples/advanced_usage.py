#!/usr/bin/env python3
"""
Advanced usage example for llamamoonlight.
"""

import json
import logging

import llamamoonlight

# Set up logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("llamamoonlight")


def on_complete(result):
    """Callback function when processing completes."""
    logger.info(f"Processing completed successfully")


def main():
    # Advanced configuration
    config = {
        "model": "llama-3-70b",
        "temperature": 0.7,
        "max_tokens": 1000,
        "top_p": 0.95,
        "use_cache": True,
        "timeout": 60,
    }

    # Initialize the client with custom configuration
    client = llamamoonlight.Client(config=config)

    # Process with callback
    logger.info("Starting processing with callback...")
    result = client.process(
        "Analyze the impact of machine learning on healthcare", callback=on_complete
    )

    # Save the result to a file
    with open("output.json", "w") as f:
        json.dump(result, f, indent=2)

    logger.info("Result saved to output.json")

    # Advanced batch processing with options
    queries = [
        "What are the ethical implications of AI?",
        "How can machine learning improve climate models?",
        "Discuss the future of autonomous vehicles",
    ]

    options = {"max_concurrency": 2, "retry_attempts": 3, "timeout": 120}

    logger.info("Starting batch processing...")
    results = client.batch_process(queries, options=options)

    # Process and analyze the results
    for i, result in enumerate(results):
        logger.info(f"Processing result {i+1}...")
        # Do something with the result


if __name__ == "__main__":
    main()
