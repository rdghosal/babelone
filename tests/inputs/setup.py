import setuptools

VERSION: str = "2.0"
REQUIRES = [
    "pydantic==2.6.2",
    "fastapi",
]


if __name__ == "__main__":
    setuptools.setup(
        name="babelone-test",
        version=VERSION,
        author="Rahul D. Ghosal",
        install_requires=REQUIRES,
        extra_requires={
            "dev": ["pytest", "hypothesis>=6.95.x"],
            "PDF": ["ReportLab>=1.2", "RXP"],
        },
        entry_points={
            "console_scripts": [
                "hello-world = timmins:hello_world",
            ]
        },
    )
