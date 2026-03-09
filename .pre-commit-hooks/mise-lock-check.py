#!/usr/bin/env python3
"""Pre-commit hook to ensure mise.lock stays in sync with mise.toml."""

import subprocess
import sys


def main():
    """Run mise lock and check if mise.lock has changed."""
    try:
        # Run mise lock
        subprocess.run(["mise", "lock"], check=True, capture_output=True)

        # Check if mise.lock has changed
        result = subprocess.run(["git", "diff", "--exit-code", "mise.lock"], capture_output=True)  # noqa: PLW1510

        if result.returncode != 0:
            print("mise.lock is out of date — stage the updated mise.lock")
            return 1

        return 0
    except subprocess.CalledProcessError as e:
        print(f"Error running mise lock: {e}")
        return 1
    except FileNotFoundError:
        print("Error: mise or git not found in PATH")
        return 1


if __name__ == "__main__":
    sys.exit(main())
