"""Installation configuration and package metadata for babelone-test."""
from setuptools import setup


if __name__ == "__main__":
    setup(
        package_name="babelone-test",
        version="v0.1.1",
        install_requires=["flask", "pydantic==2.6.1"],
        setup_requires=[],
    )