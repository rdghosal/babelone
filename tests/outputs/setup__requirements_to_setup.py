"""Installation configuration and package metadata."""
from setuptools import setup


if __name__ == "__main__":
    setup(
        install_requires=["flask", "pydantic==2.x"],
    )