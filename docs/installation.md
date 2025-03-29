# Installation Guide

## Prerequisites

- Python 3.8 or higher
- pip package manager
- For MLX support: macOS with Apple Silicon

## Standard Installation

### Install from PyPI

```bash
pip install llamamoonlight
```

### Install with Extra Dependencies

```bash
pip install llamamoonlight[all]  # Install all optional dependencies
pip install llamamoonlight[mlx]  # Install with MLX support
pip install llamamoonlight[web]  # Install with web components
```

## Development Installation

For development, clone the repository and install in editable mode:

```bash
git clone https://github.com/llamasearchai/llamamoonlight.git
cd llamamoonlight
pip install -e ".[dev]"  # Install with development dependencies
```

## Docker Installation

```bash
docker pull llamasearchai/llamamoonlight:latest
docker run -p 8000:8000 llamasearchai/llamamoonlight:latest
```

## Verification

To verify your installation:

```python
import llamamoonlight
print(llamamoonlight.__version__)
```

You should see the version number of the installed package.

## Troubleshooting

### Common Issues

1. **ImportError**: Make sure you have installed all required dependencies.
2. **Version Conflicts**: Try creating a new virtual environment.
3. **MLX Issues**: Ensure you're on Apple Silicon hardware for MLX support.

### Getting Help

If you encounter issues, please:

1. Check the [documentation](https://llamasearchai.github.io/llamamoonlight/)
2. Search for similar [issues on GitHub](https://github.com/llamasearchai/llamamoonlight/issues)
3. Ask for help in our [Discord community](https://discord.gg/llamasearch)
