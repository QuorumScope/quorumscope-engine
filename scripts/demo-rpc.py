#!/usr/bin/env python3
"""Small deterministic JSON-RPC fixture for the QuorumScope reviewer demo."""

import argparse
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


parser = argparse.ArgumentParser()
parser.add_argument("--fixture", required=True)
parser.add_argument("--port", type=int, default=18081)
args = parser.parse_args()
fixture = json.load(open(args.fixture, encoding="utf-8"))


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):  # noqa: N802 - stdlib HTTP server API
        length = int(self.headers.get("content-length", "0"))
        request = json.loads(self.rfile.read(length))
        method = request.get("method")
        if method == "getLatestLedger":
            result = {
                "id": "fixture-ledger",
                "protocolVersion": fixture["protocol_version"],
                "sequence": fixture["latest_ledger"],
            }
        elif method == "getLedgerEntries":
            result = {
                "latestLedger": fixture["latest_ledger"],
                "entries": [
                    {
                        "key": "fixture-frozen-ledger-keys",
                        "xdr": fixture["frozen_ledger_keys_xdr"],
                        "lastModifiedLedgerSeq": fixture["latest_ledger"],
                    },
                    {
                        "key": "fixture-freeze-bypass-txs",
                        "xdr": fixture["freeze_bypass_txs_xdr"],
                        "lastModifiedLedgerSeq": fixture["latest_ledger"],
                    },
                ],
            }
        else:
            self.send_error(400, f"unsupported fixture method: {method}")
            return

        body = json.dumps({"jsonrpc": "2.0", "id": request.get("id"), "result": result}).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, format, *args):  # noqa: A002 - stdlib signature
        return


ThreadingHTTPServer(("127.0.0.1", args.port), Handler).serve_forever()
