import asyncio
from .uvx import main_rs # src/lib.rs


async def async_main_py():
    """
    Async entrypoint ('main_rs' can't be used with asyncio.run directly)
    """
    try:
        code = await main_rs() # returns exit code
    except Exception as e:
        code = 1

    exit(code)


def main_py():
    """
    Sync entrypoint.

    Using asyncio allows using async rust code (via tokio).
    """
    asyncio.run(async_main_py())
