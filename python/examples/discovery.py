"""Anonymous catalogue discovery; no trading token is needed."""

from agara_sdk import AgaraClient


def main() -> None:
    with AgaraClient() as client:
        page = client.list_markets(source="agara", state="ACTIVE", limit=10)
        for market in page["markets"]:
            print(market["id"], market["question"])


if __name__ == "__main__":
    main()
