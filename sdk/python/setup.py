from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="dchat-sdk",
    version="0.1.0",
    author="dchat contributors",
    author_email="",
    description="dchat Python SDK - Build decentralized chat applications",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/dchat/dchat",
    packages=find_packages(exclude=["tests", "examples"]),
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "Topic :: Communications :: Chat",
        "Topic :: Security :: Cryptography",
        "License :: OSI Approved :: MIT License",
        "License :: OSI Approved :: Apache Software License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
    ],
    python_requires=">=3.8",
    install_requires=[
        "aiohttp>=3.9.0",
        "cryptography>=41.0.0",
        "ed25519>=1.5",
    ],
    extras_require={
        "dev": [
            "pytest>=7.4.0",
            "pytest-asyncio>=0.21.0",
            "pytest-cov>=4.1.0",
            "mypy>=1.5.0",
            "black>=23.7.0",
            "isort>=5.12.0",
            "flake8>=6.1.0",
        ],
    },
    keywords="dchat decentralized chat p2p blockchain encryption",
    project_urls={
        "Bug Reports": "https://github.com/dchat/dchat/issues",
        "Source": "https://github.com/dchat/dchat",
        "Documentation": "https://github.com/dchat/dchat/tree/main/sdk/python",
    },
)
