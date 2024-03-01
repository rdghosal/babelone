import setuptools


if __name__ == "__main__":
    setuptools.setup(
        name="babelone-test",
        version="2.0",
        author="Rahul D. Ghosal",
        install_requires=[
            "pydantic==2.6.2",
            "fastapi",
        ],
    )
