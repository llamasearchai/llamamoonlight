#!/usr/bin/env python3
"""
Basic usage example for llamamoonlight.
"""

import llamamoonlight


def main():
    # Initialize the client
    client = llamamoonlight.Client()

    # Process a query
    result = client.process("How does machine learning work?")

    # Print the result
    print("Result:")
    print(result)

    # Process multiple queries
    queries = [
        "What is deep learning?",
        "How do transformers work?",
        "Explain neural networks",
    ]

    results = client.batch_process(queries)

    # Print the results
    print("\nBatch Results:")
    for i, result in enumerate(results):
        print(f"Query {i+1}: {queries[i]}")
        print(f"Result: {result}")
        print()


if __name__ == "__main__":
    main()
