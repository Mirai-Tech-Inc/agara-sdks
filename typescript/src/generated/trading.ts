export interface paths {
    "/trade/v1/status": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Get platform status */
        get: operations["status"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/orders": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Place an order */
        post: operations["create_order"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/orders/signed": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Place a pre-signed order */
        post: operations["create_signed_order"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/orders/signed/batch": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Place a batch of pre-signed orders */
        post: operations["create_signed_order_batch"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/orders/list": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** List your orders */
        post: operations["list_orders"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/orders/{order_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Get an order */
        get: operations["get_order"];
        put?: never;
        post?: never;
        /** Cancel an order */
        delete: operations["cancel_order"];
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/orders/{order_id}/trades": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** List an order's fills */
        get: operations["get_order_trades"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/orders/cancel-all": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Schedule cancellation of current nonterminal orders */
        post: operations["cancel_all_orders"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/batches": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Submit an account batch */
        post: operations["create_batch"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/batches/{batch_hash}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Get an account batch */
        get: operations["get_batch"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/batches/{batch_hash}/supersede": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Replace a pending account batch */
        post: operations["supersede_batch"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/orderbook/{token_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Get orderbook depth */
        get: operations["agara_orderbook"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/portfolio/activities": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** List your account activity */
        get: operations["portfolio_activities"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/portfolio/bridge/withdraw/supported-assets": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** List supported withdrawal chains and assets */
        get: operations["portfolio_bridge_withdraw_supported_assets"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/portfolio/bridge/withdraw/quote": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Quote a cross-chain withdrawal */
        post: operations["portfolio_bridge_withdraw_quote"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/portfolio/open-orders/list": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** List your active orders */
        post: operations["portfolio_open_orders_list"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/portfolio/positions/list": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** List your positions */
        post: operations["portfolio_positions_list"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/portfolio/positions/split": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Split collateral into a complete set */
        post: operations["portfolio_positions_split"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/portfolio/positions/merge": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Merge a complete set into collateral */
        post: operations["portfolio_positions_merge"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/portfolio/summary": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Get portfolio summary */
        get: operations["portfolio_summary"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/portfolio/trades": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** List your fills */
        get: operations["portfolio_trades"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/portfolio/rebates": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Get pending rebate balances */
        get: operations["portfolio_rebates"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/lp-incentives": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Get LP-incentive opportunities and your standing */
        get: operations["lp_incentives"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/lp-incentives/categories": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** List categories containing current LP-incentive markets */
        get: operations["lp_incentive_categories"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/lp-incentives/closed": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** List your LP reward cycles */
        get: operations["closed_lp_incentives"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/lp-incentives/closed/categories": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** List root categories over the markets you earned LP rewards in */
        get: operations["closed_lp_incentive_categories"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/lp-incentives/earnings": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Get your LP rewards summed over trailing windows */
        get: operations["lp_incentive_earnings"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trade/v1/portfolio/pnl/realized": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Get exact realised PnL */
        get: operations["portfolio_realized_pnl"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
}
export type webhooks = Record<string, never>;
export interface components {
    schemas: {
        /** @description Acceptance response: the digest that keys the row and the workflow. */
        AccountBatchAccepted: {
            /** @description EIP-712 batch digest, 0x hex. */
            batch_hash: string;
        };
        AccountBatchStatusDto: {
            batch_hash: string;
            /** Format: date-time */
            created_at: string;
            /** Format: int64 */
            deadline_unix_seconds: number | bigint;
            /** Format: date-time */
            executed_at?: string | null;
            failure?: components["schemas"]["KnownPublicFailure"] | null;
            heals_batch_hash?: string | null;
            origin: string;
            /** Format: int64 */
            seq: number | bigint;
            status: string;
            superseded_by_batch_hash?: string | null;
            tx_hash?: string | null;
            /** Format: date-time */
            unwound_at?: string | null;
        };
        /**
         * @description Presigned submission body: typed ops plus the signed binding tuple.
         * @example {
         *       "deadline_unix_seconds": 1794700800,
         *       "ops": [
         *         {
         *           "condition_id": "0x21742633143463906290569050155826241533067272736897614950488156847949938836455",
         *           "kind": "MERGE",
         *           "market_id": "9b2f9c6e-0000-0000-0000-000000000000",
         *           "shares_micro": 1000000
         *         }
         *       ],
         *       "seq": 7,
         *       "signature": "0x4f3a8e9b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001b"
         *     }
         */
        AccountBatchSubmission: {
            /**
             * Format: int64
             * @description Signature deadline, unix seconds.
             */
            deadline_unix_seconds: number | bigint;
            /** @description The `FAILED_DIVERGENT` batch this batch heals, if any. */
            heals_batch_hash?: string | null;
            /** @description Typed ops in execution order. */
            ops: components["schemas"]["BatchOpDto"][];
            /**
             * Format: int64
             * @description Account seq the signature binds. The live seq, or the next one above it when batches are
             *     already queued on this wallet; it may not skip a seq that no batch holds.
             */
            seq: number | bigint;
            /** @description Holder signature over the batch digest, 0x hex. */
            signature: string;
        };
        AccountBatchSupersedeOutcome: {
            batch: components["schemas"]["AccountBatchStatusDto"];
            /** @enum {string} */
            outcome: "SUPERSEDED";
        } | {
            current: components["schemas"]["AccountBatchStatusDto"];
            /** @enum {string} */
            outcome: "REFUSED";
        };
        /**
         * @description Supersede body: new ops re-signed at the superseded batch's seq.
         * @example {
         *       "deadline_unix_seconds": 1794700800,
         *       "ops": [
         *         {
         *           "amount_micro": 1000000,
         *           "destination": "0x1111111111111111111111111111111111111111",
         *           "kind": "WITHDRAW"
         *         }
         *       ],
         *       "signature": "0x4f3a8e9b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001b"
         *     }
         */
        AccountBatchSupersedeSubmission: {
            /**
             * Format: int64
             * @description Signature deadline, unix seconds.
             */
            deadline_unix_seconds: number | bigint;
            /** @description Typed ops in execution order. */
            ops: components["schemas"]["BatchOpDto"][];
            /** @description Holder signature over the successor digest (same seq), 0x hex. */
            signature: string;
        };
        /** @description FE-facing orderbook snapshot for a single agara token. */
        AgaraOrderbookResponse: {
            /** @description Asks sorted ascending by price (best first). */
            asks: components["schemas"]["OrderbookLevel"][];
            /** @description Bids sorted descending by price (best first). */
            bids: components["schemas"]["OrderbookLevel"][];
            /**
             * @description Snapshot sequence number stringified — two snapshots with the
             *     same hash are byte-identical.
             * @example 1234
             */
            hash: string;
            /**
             * @description Minimum price increment for this market.
             * @example 0.01
             */
            tick_size: string;
            /**
             * @description UTC ISO-8601 timestamp stamped at response time.
             * @example 2026-05-12T10:23:45.678Z
             */
            timestamp: string;
        };
        /**
         * @description One typed intent inside an account batch, in execution order.
         *
         *     Serialized with a `kind` tag in SCREAMING_SNAKE_CASE, matching the wire
         *     contract of `POST /trade/v1/positions/batch/signed`.
         */
        BatchOpDto: {
            /**
             * @description 0x-prefixed condition id the client signed over; must equal the
             *     stored condition of `market_id`.
             */
            condition_id: string;
            /** @enum {string} */
            kind: "SPLIT";
            /**
             * Format: uuid
             * @description Market row id used for lookup and gating.
             */
            market_id: string;
            /**
             * Format: int64
             * @description Complete-set size in micro units; equals the collateral consumed.
             */
            shares_micro: number | bigint;
        } | {
            /** @description 0x-prefixed condition id the client signed over. */
            condition_id: string;
            /** @enum {string} */
            kind: "MERGE";
            /**
             * Format: uuid
             * @description Market row id used for lookup and gating.
             */
            market_id: string;
            /**
             * Format: int64
             * @description Complete-set size in micro units; equals the collateral produced.
             */
            shares_micro: number | bigint;
        } | {
            /**
             * Format: int64
             * @description Amount in micro units.
             */
            amount_micro: number | bigint;
            /** @description 0x-prefixed recipient; never the wallet's own account. */
            destination: string;
            /** @enum {string} */
            kind: "WITHDRAW";
        } | {
            /**
             * Format: int64
             * @description Must equal the paired withdrawal input, in micro units.
             */
            amount_micro: number | bigint;
            /** @enum {string} */
            kind: "ACROSS_SWAP_APPROVE";
            /** @description Allowlisted provider target that consumes the allowance. */
            spender: string;
        } | {
            /** @description The wallet's own account and Across refund address. */
            depositor: string;
            /**
             * Format: int64
             * @description Destination chain id.
             */
            destination_chain_id: number | bigint;
            /** @description Exclusive relayer or the zero address. */
            exclusive_relayer: string;
            /**
             * Format: int32
             * @description Raw offset-or-timestamp exclusivity parameter.
             */
            exclusivity_parameter: number;
            /**
             * Format: int32
             * @description Across fill deadline.
             */
            fill_deadline: number;
            /**
             * Format: int64
             * @description Base USDC input and engine debit, in micro units.
             */
            input_amount_micro: number | bigint;
            /** @description Public two-byte id, `0x` prefixed. */
            integrator_id?: string | null;
            /** @enum {string} */
            kind: "WITHDRAW_ACROSS_SWAP_DEPOSIT";
            /**
             * Format: int64
             * @description Conservative destination output in token base units.
             */
            output_amount_base_units: number | bigint;
            /** @description Destination token. */
            output_token: string;
            /**
             * Format: int32
             * @description Across quote timestamp.
             */
            quote_timestamp: number;
            /** @description Destination recipient. */
            recipient: string;
        } | {
            /** @description The wallet's own account and Across refund address. */
            depositor: string;
            /**
             * Format: int64
             * @description Destination chain id.
             */
            destination_chain_id: number | bigint;
            /** @description Exclusive relayer or the zero address. */
            exclusive_relayer: string;
            /**
             * Format: int32
             * @description Raw offset-or-timestamp exclusivity parameter.
             */
            exclusivity_deadline: number;
            /**
             * Format: int32
             * @description Across fill deadline.
             */
            fill_deadline: number;
            /**
             * Format: int64
             * @description Base USDC input and engine debit, in micro units.
             */
            input_amount_micro: number | bigint;
            /** @description Public two-byte id, `0x` prefixed. */
            integrator_id?: string | null;
            /** @enum {string} */
            kind: "WITHDRAW_ACROSS_SWAP_DEPOSIT_V3";
            /**
             * Format: int64
             * @description Conservative destination output in token base units.
             */
            output_amount_base_units: number | bigint;
            /** @description Destination token. */
            output_token: string;
            /**
             * Format: int32
             * @description Across quote timestamp.
             */
            quote_timestamp: number;
            /** @description Destination recipient. */
            recipient: string;
        } | {
            /** @description Only address permitted to deliver the Circle message, padded bytes32 hex. */
            destination_caller: string;
            /**
             * Format: int64
             * @description Destination chain id pinned to the shared CCTP policy.
             */
            destination_chain_id: number | bigint;
            /**
             * Format: int64
             * @description Base USDC input and engine debit, in micro units.
             */
            input_amount_micro: number | bigint;
            /** @description Public two-byte id, `0x` prefixed. */
            integrator_id?: string | null;
            /** @enum {string} */
            kind: "WITHDRAW_ACROSS_SWAP_CCTP";
            /**
             * Format: int64
             * @description Maximum destination fee in Base-USDC units.
             */
            max_fee_base_units: number | bigint;
            /**
             * Format: int32
             * @description Exact Circle finality threshold.
             */
            min_finality_threshold: number;
            /** @description Destination recipient. */
            recipient: string;
        };
        /**
         * @description Outcome of one order in a batch, mirroring the single-order endpoint. An
         *     accepted order carries the same fields as [`CreateClobOrderResponse`]; an
         *     expected per-order rejection carries canonical public failure metadata.
         *     Tagged on `outcome` rather than `status` because the accepted body already
         *     has its own `status` field.
         */
        BatchSignedOrderOutcome: (components["schemas"]["CreateClobOrderResponse"] & {
            /** @enum {string} */
            outcome: "accepted";
        }) | {
            /** @description Registry-backed code, public detail, and recovery guidance. */
            failure: components["schemas"]["KnownPublicFailure"];
            /** @enum {string} */
            outcome: "rejected";
        };
        /** @description One batch entry, tagged with its position in the submitted array. */
        BatchSignedOrderResult: components["schemas"]["BatchSignedOrderOutcome"] & {
            /** @description Zero-based index into the submitted `orders` array. */
            index: number;
        };
        /** @description CLOB cancel-all response. */
        CancelAllClobOrdersResponse: {
            /**
             * Format: date-time
             * @description Response timestamp.
             */
            as_of: string;
            /** @description Pending async operation. */
            pending_operation: components["schemas"]["ClobOrderPendingOperation"];
            /** @description Wallet identifiers included in the cancellation request. */
            wallet_ids: string[];
        };
        /** @description CLOB order cancellation response. */
        CancelClobOrderResponse: {
            /**
             * Format: date-time
             * @description Response timestamp.
             */
            as_of: string;
            /**
             * Format: uuid
             * @description Internal order UUID.
             */
            order_id: string;
            /** @description Pending async operation. */
            pending_operation: components["schemas"]["ClobOrderPendingOperation"];
        };
        /**
         * @description Accepted pending operation.
         * @enum {string}
         */
        ClobOrderPendingOperation: "SUBMIT" | "CANCEL" | "CANCEL_ALL";
        /** @description Local CLOB order response. */
        ClobOrderResponse: {
            /** @description Market display metadata keyed by outcome token id. */
            markets: {
                [key: string]: components["schemas"]["MarketMeta"];
            };
            /** @description Order row. */
            order: components["schemas"]["Order"];
        };
        /**
         * @description Local order status.
         * @enum {string}
         */
        ClobOrderStatus: "PENDING" | "SUBMITTING" | "OPEN" | "PARTIALLY_FILLED" | "MATCHED" | "CANCELLED" | "EXPIRED" | "REJECTED" | "FAILED";
        /**
         * @description CLOB order time in force.
         * @enum {string}
         */
        ClobOrderTimeInForce: "FAK" | "FOK" | "GTC" | "GTD";
        /**
         * @description CLOB order type.
         * @enum {string}
         */
        ClobOrderType: "LIMIT" | "MARKET";
        /**
         * @description Local CLOB order list request.
         * @example {
         *       "limit": 100
         *     }
         */
        ClobOrdersListRequest: {
            /**
             * @description Opaque cursor from a prior response's `next_cursor`; omit for the first
             *     page.
             */
            cursor?: string | null;
            /** @description Page limit. */
            limit?: components["schemas"]["PortfolioListLimit"] | null;
        };
        /** @description Local CLOB order list response. */
        ClobOrdersListResponse: {
            /**
             * Format: date-time
             * @description Response timestamp.
             */
            as_of: string;
            /** @description Market display metadata keyed by outcome token id. */
            markets: {
                [key: string]: components["schemas"]["MarketMeta"];
            };
            /** @description Order rows. */
            orders: components["schemas"]["Order"][];
            /** @description Keyset pagination state. */
            pagination: components["schemas"]["CursorPagination_PortfolioListLimit"];
        };
        ClosedLpIncentiveMarket: {
            /** @description User-facing root-category label, or `null` when the parent event has no active category. */
            category_label?: string | null;
            /** @description Root-category slug, or `null` when the parent event has no active category. */
            category_slug?: string | null;
            /** @description Resolution time as RFC 3339, or `null` while the market is live. */
            closed_at?: string | null;
            /** @description Whether settlement has credited this cycle. */
            credited: boolean;
            /** @description Rewards credited to the caller in this cycle. */
            earned_micro: string;
            /** @description The reward day this row covers. */
            epoch_date: string;
            /**
             * Format: double
             * @description The caller's score in this cycle.
             */
            epoch_score: number;
            /** @description Parent event slug, for linking to the market's event page. */
            event_slug: string;
            /** @description Parent event title. */
            event_title: string;
            /** @description Whether the caller has traded this market at least once. */
            has_traded: boolean;
            /** @description Agara market UUID. */
            market_id: string;
            /**
             * Format: double
             * @description Total market score in this cycle.
             */
            market_total_score: number;
            /** @description This cycle's reward pool. */
            pool_micro: string;
            /** @description This cycle's pro-rata payout before the minimum payout is applied. */
            potential_payout_micro: string;
            /** @description Market question. */
            question: string;
        };
        /** @enum {string} */
        ClosedLpIncentiveSortBy: "date" | "earned";
        ClosedLpIncentivesResponse: {
            /**
             * @description Always `true`: every authenticated caller sees their reward rows.
             *     Retained for API compatibility.
             */
            eligible: boolean;
            /**
             * Format: int32
             * @description Requested page size.
             */
            limit: number;
            /** @description The caller's credited LP reward cycles in this page. */
            markets: components["schemas"]["ClosedLpIncentiveMarket"][];
            /**
             * Format: int32
             * @description 1-based page number this response covers.
             */
            page: number;
            /**
             * Format: int64
             * @description Total rows matching the filters across all pages.
             */
            total: number | bigint;
        };
        /** @description Market condition identifier. */
        ConditionId: string;
        /**
         * @description Market display metadata for condition-level rows (splits), keyed in
         *     responses by condition id. Logo resolution is condition-level (market
         *     image, then event icon) — outcome logos are team-dependent and would
         *     flap across requests under the outcome-keyed store's last-row-wins.
         */
        ConditionMeta: {
            /** @description Raw market display passthrough. */
            display: components["schemas"]["ConditionMetaDisplay"];
            /** @description Event slug. */
            event_slug: string;
            /** @description Logo URL. */
            logo_url?: string | null;
            /**
             * Format: uuid
             * @description Canonical market UUID.
             */
            market_id: string;
            /** @description Market title. */
            market_title: string;
        };
        /**
         * @description Raw CMS display JSONB for a condition-level sidecar entry — market only. A
         *     condition aggregates every outcome leg, so no single outcome display applies
         *     (and an outcome blob would flap under the store's last-row-wins, like logos).
         */
        ConditionMetaDisplay: {
            /** @description Raw market-level display blob as stored by the CMS. */
            market: Record<string, never>;
        };
        /**
         * @description CLOB order creation request body.
         * @example {
         *       "price_micro": "600000",
         *       "shares_micro": "1000000",
         *       "side": "BUY",
         *       "time_in_force": "GTC",
         *       "token_id": "21742633143463906290569050155826241533067272736897614950488156847949938836455",
         *       "type": "LIMIT"
         *     }
         */
        CreateClobOrderRequest: {
            /**
             * @description Collateral amount to spend, in micro units. The size for a `MARKET`
             *     BUY. Exactly one of `collateral_amount_micro` and `shares_micro` may
             *     be set.
             */
            collateral_amount_micro?: components["schemas"]["MicroAmount"] | null;
            /**
             * Format: int64
             * @description Expiration in Unix seconds. Required for `GTD` and must be at least
             *     30 seconds in the future; omit it for every other time-in-force.
             * @example 1794700800
             */
            expiration_unix_seconds?: (number | bigint) | null;
            /**
             * @description Post-only flag. `LIMIT` only; the order is rejected instead of
             *     crossing the spread. Defaults to `false`.
             */
            post_only?: boolean | null;
            /** @description Limit price in micro units. Required for `LIMIT`; rejected for `MARKET`. */
            price_micro?: components["schemas"]["MicroAmount"] | null;
            /**
             * @description Shares amount in micro units. The size for a `LIMIT` order or a
             *     `MARKET` SELL. Exactly one of `collateral_amount_micro` and
             *     `shares_micro` may be set.
             */
            shares_micro?: components["schemas"]["MicroAmount"] | null;
            /** @description Order side. */
            side: components["schemas"]["Side"];
            /** @description Time in force. */
            time_in_force: components["schemas"]["ClobOrderTimeInForce"];
            /** @description Outcome token id. */
            token_id: components["schemas"]["TokenId"];
            /** @description Order type. */
            type: components["schemas"]["ClobOrderType"];
        };
        /** @description CLOB order accepted response. */
        CreateClobOrderResponse: {
            /**
             * Format: date-time
             * @description Response timestamp.
             */
            as_of: string;
            /**
             * Format: uuid
             * @description Internal order UUID.
             */
            order_id: string;
            /** @description Pending async operation. */
            pending_operation: components["schemas"]["ClobOrderPendingOperation"];
            /** @description Order source. */
            source: components["schemas"]["Exchange"];
            /** @description Local order status. */
            status: components["schemas"]["ClobOrderStatus"];
        };
        /**
         * @description Batched pre-signed order creation request. Expected per-order validation
         *     failures preserve the remaining entries; shared wallet, provider, and
         *     storage failures reject the request.
         */
        CreateSignedClobOrderBatchRequest: {
            /**
             * @description The pre-signed orders to submit: at least 1, at most 32. An empty
             *     array or more than 32 orders is rejected outright with `422`.
             */
            orders: components["schemas"]["CreateSignedClobOrderRequest"][];
        };
        /**
         * @description Batched pre-signed order response. `results` is in request order, one
         *     entry per submitted order.
         */
        CreateSignedClobOrderBatchResponse: {
            /**
             * Format: date-time
             * @description Response timestamp.
             */
            as_of: string;
            /** @description Per-order outcomes, in request order. */
            results: components["schemas"]["BatchSignedOrderResult"][];
        };
        /**
         * @description CLOB order creation request body for pre-signed (bot/server-key)
         *     submissions. Carries the EIP-712 envelope fields the bot signed
         *     over so the router can recover and validate the signature without
         *     re-signing.
         * @example {
         *       "builder": "0x0000000000000000000000000000000000000000000000000000000000000000",
         *       "chain_token_id": "21742633143463906290569050155826241533067272736897614950488156847949938836455",
         *       "maker": "0x1111111111111111111111111111111111111111",
         *       "maker_amount": "600000",
         *       "metadata": "0x0000000000000000000000000000000000000000000000000000000000000000",
         *       "order_hash": "0xfe738cc74603ab4700a9e2b28a3d3e2d2f4f2b6b1b9b3a5b7c9d1e3f5a7b9c1d",
         *       "post_only": false,
         *       "price_micro": "600000",
         *       "salt": "84629103847263918473",
         *       "shares_micro": "1000000",
         *       "side": "BUY",
         *       "side_u8": 0,
         *       "signature": "0x4f3a8e9b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001b",
         *       "taker_amount": "1000000",
         *       "time_in_force": "GTC",
         *       "timestamp": "0",
         *       "token_id": "21742633143463906290569050155826241533067272736897614950488156847949938836455",
         *       "type": "LIMIT"
         *     }
         */
        CreateSignedClobOrderRequest: {
            /** @description Builder code (0x-prefixed bytes32; usually zero). */
            builder: string;
            /**
             * @description CTF position id as the u256 value the bot signed over (decimal
             *     or `0x`-hex string). Must encode the same numeric value as
             *     `token_id` above — the router validates they match. Carried
             *     separately because the lookup `token_id` is a free-form string
             *     whose encoding isn't guaranteed to match what the bot used in
             *     the EIP-712 envelope.
             */
            chain_token_id: string;
            /**
             * Format: int64
             * @description Expiration in Unix seconds. Required for `GTD` and must be at least
             *     30 seconds in the future; omit it for every other time-in-force.
             * @example 1794700800
             */
            expiration_unix_seconds?: (number | bigint) | null;
            /** @description Maker address — must equal the wallet's deposit-wallet address. */
            maker: string;
            /** @description Maker amount (u256 decimal string in chain micro units). */
            maker_amount: string;
            /** @description Metadata hash (0x-prefixed bytes32; usually zero). */
            metadata: string;
            /** @description 0x-prefixed keccak256 of the EIP-712 typed-data digest. */
            order_hash: string;
            /**
             * @description Post-only flag. The order is rejected instead of crossing the spread.
             *     Defaults to `false`.
             */
            post_only?: boolean | null;
            /** @description Limit price in micro units. */
            price_micro?: components["schemas"]["MicroAmount"] | null;
            /** @description Per-order salt (u256 decimal string). Use a fresh value per order. */
            salt: string;
            /** @description Shares amount in micro units. */
            shares_micro?: components["schemas"]["MicroAmount"] | null;
            /** @description Order side. */
            side: components["schemas"]["Side"];
            /**
             * Format: int32
             * @description Side encoded as the contract's `uint8`: 0 = BUY, 1 = SELL.
             */
            side_u8: number;
            /** @description 0x-prefixed 65-byte (r||s||v) signature from the user's EOA. */
            signature: string;
            /** @description Taker amount (u256 decimal string). */
            taker_amount: string;
            /** @description Time in force. */
            time_in_force: components["schemas"]["ClobOrderTimeInForce"];
            /** @description Order timestamp (u256 decimal string; usually 0). */
            timestamp: string;
            /** @description Outcome token id. */
            token_id: components["schemas"]["TokenId"];
            /** @description Order type — only `LIMIT` is supported on this endpoint. */
            type: components["schemas"]["ClobOrderType"];
        };
        /**
         * @description Shared cursor-paginated response envelope. Carries the next page's opaque
         *     cursor (absent when the page is the last one) and the applied page size.
         *     Generic over the limit newtype so every list endpoint serializes the same
         *     `{ next_cursor, limit }` shape regardless of which limit type it applies;
         *     utoipa inlines the concrete limit schema at each response field.
         */
        CursorPagination_PortfolioActivityLimit: {
            /**
             * Format: int32
             * @description Activities limit: 1 to 500, defaulting to 50.
             * @default 50
             * @example 50
             */
            limit: number;
            /** @description Opaque cursor for the next page; `null` when no more rows remain. */
            next_cursor?: string | null;
        };
        /**
         * @description Shared cursor-paginated response envelope. Carries the next page's opaque
         *     cursor (absent when the page is the last one) and the applied page size.
         *     Generic over the limit newtype so every list endpoint serializes the same
         *     `{ next_cursor, limit }` shape regardless of which limit type it applies;
         *     utoipa inlines the concrete limit schema at each response field.
         */
        CursorPagination_PortfolioListLimit: {
            /**
             * Format: int32
             * @description Portfolio list limit: 1 to 500, defaulting to 500.
             * @default 500
             * @example 100
             */
            limit: number;
            /** @description Opaque cursor for the next page; `null` when no more rows remain. */
            next_cursor?: string | null;
        };
        /**
         * @description Event display metadata sidecar entry, keyed in responses by event slug.
         *     Event display is deduped here rather than nested per-outcome in `markets`,
         *     where the same (potentially large) blob would repeat for every outcome.
         */
        EventMeta: {
            /** @description Raw event display passthrough. */
            display: components["schemas"]["EventMetaDisplay"];
            /** @description Event title. */
            event_title: string;
        };
        /** @description Raw CMS display JSONB for an event sidecar entry. */
        EventMetaDisplay: {
            /** @description Raw event-level display blob as stored by the CMS. */
            event: Record<string, never>;
        };
        /**
         * @description The AGARA exchange identifier.
         * @enum {string}
         */
        Exchange: "AGARA";
        /**
         * @description One wallet's single-sided view of a fill — the shared item type for
         *     `/trade/v1/portfolio/trades` and `/trade/v1/orders/{id}/trades`.
         */
        Fill: {
            /**
             * @description Exchange this fill settled on. Drives FE behaviour like which
             *     block explorer to link `transaction_hash` against.
             */
            exchange: components["schemas"]["Exchange"];
            /**
             * Format: date-time
             * @description Execution timestamp.
             */
            executed_at: string;
            /** @description Fee charged to this wallet in micro-collateral units; `0` means no fee. */
            fee_micro: components["schemas"]["MicroAmount"];
            /** @description The engine fill identifier and stream/REST join key, serialized as a string because it can exceed the JavaScript safe-integer range. */
            fill_id: string;
            /** @description The UUID of the order that placed this side of the fill. */
            order_id: components["schemas"]["OrderId"];
            /** @description This wallet's fill price (micro). */
            price_micro: components["schemas"]["MicroAmount"];
            /** @description Whether the order was the maker or taker in this fill. */
            role: components["schemas"]["FillRole"];
            /** @description Shares filled (micro). */
            shares_micro: components["schemas"]["MicroAmount"];
            /** @description This wallet's side of the fill. */
            side: components["schemas"]["Side"];
            /** @description Trade lifecycle status. */
            status: components["schemas"]["TradeStatus"];
            /** @description Outcome token id. */
            token_id: components["schemas"]["TokenId"];
            /** @description The AGARA trade row identifier. */
            trade_id: components["schemas"]["TradeId"];
            /** @description Settlement transaction hash; `None` until settled. */
            transaction_hash?: string | null;
        };
        /**
         * @description Which leg of a fill the wallet's order was on.
         * @enum {string}
         */
        FillRole: "MAKER" | "TAKER";
        /** @description Closed producer form of the static edge problem. */
        KnownEdgeProblem: {
            /** @description Registered machine code. */
            code: components["schemas"]["KnownProblemCode"];
            /** @description Optional allowlisted context. */
            detail?: string | null;
            /** @description Canonical recovery variant. */
            recovery: components["schemas"]["KnownRecovery"];
            /**
             * Format: int32
             * @description Canonical HTTP status.
             */
            status: number;
            /** @description Canonical safe title. */
            title: string;
            /** @description Stable problem type URN. */
            type: string;
        };
        KnownFieldError: {
            /** @enum {string} */
            code: "invalid_value";
            /** @enum {string} */
            message: "Invalid request value.";
            path: (string | number)[];
        } | {
            /** @enum {string} */
            code: "additional_errors";
            /** @enum {string} */
            message: "Additional validation errors were omitted.";
            path: (string | number)[];
        };
        /** @description Closed producer form of origin Problem Details. */
        KnownOriginProblemDetails: {
            /** @description Registered machine code. */
            code: components["schemas"]["KnownProblemCode"];
            /** @description Optional allowlisted context. */
            detail?: string | null;
            /** @description Bounded validation failures. */
            field_errors?: components["schemas"]["KnownFieldError"][] | null;
            /** @description Canonical recovery variant. */
            recovery: components["schemas"]["KnownRecovery"];
            /** @description Origin-generated request UUID. */
            request_id: string;
            /**
             * Format: int32
             * @description Canonical HTTP status.
             */
            status: number;
            /** @description Canonical safe title. */
            title: string;
            /** @description Stable problem type URN. */
            type: string;
        };
        /**
         * @description A problem code that an Agara producer may emit.
         * @enum {string}
         */
        KnownProblemCode: "account_frozen" | "account_quarantined" | "batch_in_flight" | "batch_not_found" | "batch_superseded" | "category_not_found" | "chain_execution_failed" | "credentials_missing" | "dependency_credentials_rejected" | "dependency_invalid_response" | "dependency_outcome_unknown" | "dependency_unavailable" | "duplicate_order" | "endpoint_retired" | "event_not_found" | "feature_not_configured" | "forbidden_scope" | "group_not_found" | "identity_token_expired" | "insufficient_balance" | "insufficient_shares" | "internal_error" | "invalid_batch" | "invalid_identity_token" | "invalid_message" | "invalid_path_parameter" | "invalid_personal_access_token" | "invalid_service_credential" | "invalid_signature" | "lp_payout_not_recoverable" | "lp_payout_run_not_found" | "malformed_json" | "malformed_websocket_message" | "market_closed" | "market_not_found" | "method_not_allowed" | "order_expiration_too_soon" | "order_funding_failed" | "order_not_cancellable" | "order_not_fillable" | "order_not_found" | "order_price_out_of_range" | "order_size_above_maximum" | "order_size_below_minimum" | "order_submission_failed" | "order_value_above_maximum" | "order_value_below_minimum" | "pnl_not_ready" | "pnl_unavailable" | "post_only_would_cross" | "price_feed_not_found" | "price_service_not_configured" | "price_stream_capacity_exceeded" | "price_stream_not_configured" | "price_temporarily_unavailable" | "profile_not_found" | "rate_limited" | "request_body_too_large" | "route_not_found" | "security_not_found" | "sequence_conflict" | "server_restarting" | "settlement_divergent" | "settlements_pending" | "signer_authorization_required" | "slow_consumer" | "stream_subject_not_found" | "stream_subject_unavailable" | "subscription_limit_exceeded" | "token_not_found" | "trading_day_not_found" | "unsupported_media_type" | "unsupported_websocket_data" | "validation_failed" | "value_cap_exceeded" | "venue_not_found" | "wallet_not_registered" | "wallet_setup_failed" | "wallet_setup_incomplete" | "wallet_setup_outcome_unknown" | "websocket_message_too_large" | "websocket_protocol_error" | "websocket_upgrade_required" | "wrong_credential_type";
        /** @description Closed producer form of protocol-neutral failure details. */
        KnownPublicFailure: {
            /** @description Registered machine code. */
            code: components["schemas"]["KnownProblemCode"];
            /** @description Optional allowlisted context. */
            detail?: string | null;
            /** @description Canonical recovery variant. */
            recovery: components["schemas"]["KnownRecovery"];
            /** @description Canonical title from metadata. */
            title: string;
        };
        /** @description A recovery value a producer may emit. */
        KnownRecovery: {
            /** @enum {string} */
            strategy: "none";
        } | {
            /** @enum {string} */
            strategy: "retry";
        } | {
            /**
             * Format: int32
             * @description Delay in seconds.
             */
            after_seconds: number;
            /** @enum {string} */
            strategy: "retry_after";
        } | {
            /** @enum {string} */
            strategy: "refresh_identity_token";
        } | {
            /** @description Existing durable resource. */
            resource: components["schemas"]["RecoveryResource"];
            /** @enum {string} */
            strategy: "check_status";
        };
        LpIncentiveCategoriesResponse: {
            /** @description Active root categories containing current LP-incentive markets. */
            categories: components["schemas"]["LpIncentiveCategory"][];
        };
        LpIncentiveCategory: {
            /** @description User-facing root-category label. */
            label: string;
            /**
             * Format: int64
             * @description Distinct current incentive markets assigned under the root.
             */
            market_count: number | bigint;
            /** @description Stable root-category slug. */
            slug: string;
        };
        LpIncentiveEarningsResponse: {
            /** @description All rewards ever credited to the caller. */
            all_time_micro: string;
            /** @description Rewards credited over the current reward date and the 29 prior ones. */
            last_30_days_micro: string;
            /** @description Rewards credited over the current reward date and the 6 prior ones. */
            last_7_days_micro: string;
            /** @description Rewards credited for the previous reward date, in micro-collateral units. */
            last_cycle_micro: string;
        };
        LpIncentiveMarket: {
            /** @description Event close time as RFC 3339, if set. */
            close_time?: string | null;
            /**
             * Format: double
             * @description The caller's current epoch score: their summed per-sample shares.
             */
            epoch_score: number;
            /** @description Parent event slug, for linking to the market's event page. */
            event_slug: string;
            /** @description Parent event title. */
            event_title: string;
            /** @description Market-level artwork, or `null` when none is configured. */
            logo_url?: string | null;
            /** @description Agara market UUID. */
            market_id: string;
            /**
             * Format: double
             * @description Total epoch score across all makers (= the market's scored-sample count).
             */
            market_total_score: number;
            /**
             * Format: int32
             * @description Maximum qualifying distance from the midpoint, in micro-probability.
             */
            max_spread_micro: number;
            /** @description Minimum qualifying resting size, in micro-shares. */
            min_shares_micro: string;
            /** @description User-facing lifecycle bucket for this market. */
            phase: components["schemas"]["LpIncentiveMarketPhase"];
            /** @description This epoch's reward pool for the market, in micro-collateral units. */
            pool_micro: string;
            /** @description Projected payout in micro-collateral units before the minimum payout is applied. */
            projected_payout_micro: string;
            /** @description Market question. */
            question: string;
        };
        /** @enum {string} */
        LpIncentiveMarketPhase: "ACTIVE" | "UPCOMING";
        /** @enum {string} */
        LpIncentiveSortBy: "market" | "close_time" | "max_spread" | "min_shares" | "reward_pool" | "my_share" | "projected_payout";
        /** @enum {string} */
        LpIncentiveSortOrder: "asc" | "desc";
        LpIncentivesResponse: {
            /**
             * @description Always `true`: every caller sees the market list. Retained for API
             *     compatibility.
             */
            eligible: boolean;
            /** @description Current Eastern reward day. */
            epoch_date: string;
            /** @description Exclusive end of the current reward day. */
            epoch_ends_at: string;
            /** @description Active and upcoming LP-participating markets with the caller's current-epoch standing. */
            markets: components["schemas"]["LpIncentiveMarket"][];
        };
        /** @description Market display metadata sidecar entry, keyed in responses by outcome token id. */
        MarketMeta: {
            /** @description Raw market + outcome display passthrough. */
            display: components["schemas"]["MarketMetaDisplay"];
            /** @description Event slug. */
            event_slug: string;
            /** @description Logo URL. */
            logo_url?: string | null;
            /**
             * Format: uuid
             * @description Canonical market UUID.
             */
            market_id: string;
            /** @description Market title. */
            market_title: string;
            /**
             * @description Neg-risk group id; `Some` marks group membership. Members are excluded
             *     from the auto-redeem sweep, so their holdings never enter redemption.
             */
            neg_risk_id?: string | null;
            /** @description Outcome name. */
            outcome_name: string;
            /**
             * @description Market lifecycle state (`enum_agara_market_states`). Passed through as a
             *     string so a newly added state can't fail serialization here.
             */
            state: string;
        };
        /**
         * @description Raw CMS display JSONB for a market-outcome sidecar entry: the escape hatch
         *     for CMS display keys beyond the resolved fields. Best-effort, unversioned,
         *     read defensively.
         */
        MarketMetaDisplay: {
            /** @description Raw market-level display blob as stored by the CMS. */
            market: Record<string, never>;
            /** @description Raw outcome-level display blob as stored by the CMS. */
            outcome: Record<string, never>;
        };
        /**
         * Format: int64
         * @description Micro-scaled signed amount. Serialized as a string; deserialized from a string or a JSON number.
         * @example 1000000
         */
        MicroAmount: string;
        /**
         * @description On-the-wire order row — the trimmed ledger projection served by the
         *     order-list, single-order, and portfolio open-orders endpoints.
         */
        Order: {
            /** @description Average fill price in micro units. */
            avg_fill_price_micro?: components["schemas"]["MicroAmount"] | null;
            /**
             * Format: date-time
             * @description When the user requested cancellation, if a durable intent is recorded.
             */
            cancel_requested_at?: string | null;
            /** @description Collateral amount in micro units. */
            collateral_amount_micro?: components["schemas"]["MicroAmount"] | null;
            /** @description Market condition id. */
            condition_id?: components["schemas"]["ConditionId"] | null;
            /**
             * Format: date-time
             * @description Creation timestamp.
             */
            created_at: string;
            /** @description Source exchange. */
            exchange: components["schemas"]["Exchange"];
            /**
             * Format: date-time
             * @description Expiration timestamp.
             */
            expiration: string;
            /** @description Safe terminal-failure classification. */
            failure?: components["schemas"]["KnownPublicFailure"] | null;
            /**
             * Format: uuid
             * @description Internal order UUID.
             */
            internal_id: string;
            /** @description Whether the order is terminal, including a partially filled FAK with no live remainder. */
            is_terminal: boolean;
            /** @description Original shares amount in micro units. */
            original_size_micro?: components["schemas"]["MicroAmount"] | null;
            /** @description Limit price in micro units. */
            price_micro?: components["schemas"]["MicroAmount"] | null;
            /** @description Order side. */
            side: components["schemas"]["Side"];
            /** @description Total filled shares in micro units. */
            size_matched_micro: components["schemas"]["MicroAmount"];
            /** @description Local status. */
            status: components["schemas"]["ClobOrderStatus"];
            /** @description Outcome token id. */
            token_id: components["schemas"]["TokenId"];
            /** @description Order type. */
            type: components["schemas"]["ClobOrderType"];
        };
        /** @description Order identifier. */
        OrderId: string;
        /** @description Trades-for-an-order response body. */
        OrderTradesResponse: {
            /**
             * Format: date-time
             * @description Response timestamp.
             */
            as_of: string;
            /**
             * @description This order's fills, newest-first. Same `Fill` shape as
             *     `/trade/v1/portfolio/trades`.
             */
            trades: components["schemas"]["Fill"][];
        };
        OrderbookLevel: {
            /**
             * Format: double
             * @description Price in collateral units per share, not micro-units.
             * @example 0.6
             */
            price: number;
            /**
             * Format: double
             * @description Size in shares, not micro-shares.
             * @example 100
             */
            size: number;
        };
        /** @description Canonical AGARA activity response. */
        PortfolioActivitiesResponse: {
            /** @description Activities sorted by timestamp and internal reference UUID descending. */
            activities: components["schemas"]["PortfolioActivity"][];
            /**
             * Format: date-time
             * @description Database read time for this page.
             */
            as_of: string;
            /** @description Current-page condition metadata keyed by condition id. */
            conditions: {
                [key: string]: components["schemas"]["ConditionMeta"];
            };
            /** @description Current-page event metadata keyed by event slug. */
            events: {
                [key: string]: components["schemas"]["EventMeta"];
            };
            /** @description Current-page market metadata keyed by outcome token id. */
            markets: {
                [key: string]: components["schemas"]["MarketMeta"];
            };
            /** @description Current-data keyset pagination state. */
            pagination: components["schemas"]["CursorPagination_PortfolioActivityLimit"];
        };
        /** @description Canonical AGARA activity. */
        PortfolioActivity: (components["schemas"]["PortfolioOrderActivity"] & {
            /** @enum {string} */
            type: "ORDER";
        }) | (components["schemas"]["PortfolioSplitActivity"] & {
            /** @enum {string} */
            type: "SPLIT";
        }) | (components["schemas"]["PortfolioMergeActivity"] & {
            /** @enum {string} */
            type: "MERGE";
        }) | (components["schemas"]["PortfolioRedeemActivity"] & {
            /** @enum {string} */
            type: "REDEEM";
        }) | (components["schemas"]["PortfolioDepositActivity"] & {
            /** @enum {string} */
            type: "DEPOSIT";
        }) | (components["schemas"]["PortfolioWithdrawalActivity"] & {
            /** @enum {string} */
            type: "WITHDRAWAL";
        }) | (components["schemas"]["PortfolioLpPayoutActivity"] & {
            /** @enum {string} */
            type: "LP_PAYOUT";
        });
        /** @description Portfolio activity identifier. */
        PortfolioActivityId: string;
        /**
         * @description Bridge address type for a supported asset.
         * @enum {string}
         */
        PortfolioBridgeAddressType: "EVM" | "BTC" | "SOL";
        /** @description Bridge base-unit amount. */
        PortfolioBridgeBaseUnitAmount: string;
        /** @description Bridge chain identifier. */
        PortfolioBridgeChainId: string;
        /** @description Bridge deposit quote response body. */
        PortfolioBridgeDepositQuoteResponse: {
            /**
             * Format: int64
             * @description Estimated checkout time in milliseconds.
             */
            est_checkout_time_ms: number | bigint;
            /** @description Estimated fee breakdown. */
            est_fee_breakdown: components["schemas"]["PortfolioBridgeEstimatedFeeBreakdown"];
            /** @description Estimated input value in USD. */
            est_input_usd: components["schemas"]["PortfolioDecimalString"];
            /** @description Estimated output value in USD. */
            est_output_usd: components["schemas"]["PortfolioDecimalString"];
            /** @description Estimated destination token amount in base units. */
            est_to_token_base_unit: components["schemas"]["PortfolioBridgeBaseUnitAmount"];
            /** @description Bridge quote id. */
            quote_id: components["schemas"]["PortfolioBridgeQuoteId"];
        };
        /** @description Bridge deposit quote fee breakdown. */
        PortfolioBridgeEstimatedFeeBreakdown: {
            /** @description App fee label. */
            app_fee_label: string;
            /** @description App fee percent. */
            app_fee_percent: components["schemas"]["PortfolioDecimalString"];
            /** @description App fee in USD. */
            app_fee_usd: components["schemas"]["PortfolioDecimalString"];
            /** @description Fill cost percent. */
            fill_cost_percent: components["schemas"]["PortfolioDecimalString"];
            /** @description Fill cost in USD. */
            fill_cost_usd: components["schemas"]["PortfolioDecimalString"];
            /** @description Gas fee in USD. */
            gas_usd: components["schemas"]["PortfolioDecimalString"];
            /** @description Maximum slippage. */
            max_slippage: components["schemas"]["PortfolioDecimalString"];
            /** @description Minimum received amount. */
            min_received: components["schemas"]["PortfolioDecimalString"];
            /** @description Swap impact. */
            swap_impact: components["schemas"]["PortfolioDecimalString"];
            /** @description Swap impact in USD. */
            swap_impact_usd: components["schemas"]["PortfolioDecimalString"];
            /** @description Total impact. */
            total_impact: components["schemas"]["PortfolioDecimalString"];
            /** @description Total impact in USD. */
            total_impact_usd: components["schemas"]["PortfolioDecimalString"];
        };
        /** @description Bridge quote identifier. */
        PortfolioBridgeQuoteId: string;
        /** @description Bridge recipient address. */
        PortfolioBridgeRecipientAddress: string;
        /** @description Bridge-supported asset. */
        PortfolioBridgeSupportedAsset: {
            /** @description Address type to use for this asset. */
            address_type: components["schemas"]["PortfolioBridgeAddressType"];
            /** @description Source chain id. */
            chain_id: components["schemas"]["PortfolioBridgeChainId"];
            /** @description Source chain name. */
            chain_name: string;
            /** @description Minimum checkout amount in USD. */
            min_checkout_usd: components["schemas"]["PortfolioDecimalString"];
            /** @description Source token. */
            token: components["schemas"]["PortfolioBridgeSupportedToken"];
        };
        /** @description Bridge supported-assets response body. */
        PortfolioBridgeSupportedAssetsResponse: {
            /** @description SDK-provided bridge note. */
            note?: string | null;
            /** @description SDK-provided supported assets. */
            supported_assets: components["schemas"]["PortfolioBridgeSupportedAsset"][];
            /** @description Deposit wallet address. */
            wallet_address: string;
        };
        /** @description Bridge-supported token. */
        PortfolioBridgeSupportedToken: {
            /** @description Token address. */
            address: components["schemas"]["PortfolioBridgeTokenAddress"];
            /**
             * Format: int32
             * @description Token decimals.
             */
            decimals: number;
            /** @description Token name. */
            name: string;
            /** @description Token symbol. */
            symbol: string;
        };
        /** @description Bridge token address. */
        PortfolioBridgeTokenAddress: string;
        /**
         * @description Bridge withdraw quote request body.
         * @example {
         *       "from_amount_base_unit": "1000000",
         *       "recipient_address": "0x1111111111111111111111111111111111111111",
         *       "to_chain_id": "42161",
         *       "to_token_address": "0xaf88d065e77c8cC2239327C5EDb3A432268e5831"
         *     }
         */
        PortfolioBridgeWithdrawQuoteRequest: {
            /** @description Source collateral amount in base units. */
            from_amount_base_unit: components["schemas"]["PortfolioBridgeBaseUnitAmount"];
            /** @description Destination recipient address. */
            recipient_address: components["schemas"]["PortfolioBridgeRecipientAddress"];
            /** @description Destination chain id. */
            to_chain_id: components["schemas"]["PortfolioBridgeChainId"];
            /** @description Destination token address. */
            to_token_address: components["schemas"]["PortfolioBridgeTokenAddress"];
        };
        /** @description Decimal value serialized as a string. */
        PortfolioDecimalString: string;
        /** @description External deposit activity row. */
        PortfolioDepositActivity: {
            /** @description Positive magnitude; the variant conveys direction. */
            amount_micro: components["schemas"]["MicroAmount"];
            /**
             * Format: int64
             * @description Chain the transfer settled on.
             */
            chain_id: number | bigint;
            /** @description On-chain transfer peer. */
            counterparty_address: string;
            /**
             * Format: date-time
             * @description Ledger creation timestamp.
             */
            created_at: string;
            /** @description Exchange this activity occurred on. */
            exchange: components["schemas"]["Exchange"];
            /** @description Namespaced activity id. */
            id: components["schemas"]["PortfolioActivityId"];
            /** @description Lowercased collateral token address. */
            token_address: string;
            /** @description Settlement transaction hash. */
            tx_hash: string;
        };
        /**
         * Format: int32
         * @description Portfolio list limit: 1 to 500, defaulting to 500.
         * @default 500
         * @example 100
         */
        PortfolioListLimit: number;
        /** @description Confirmed liquidity-provider reward payout. */
        PortfolioLpPayoutActivity: {
            /** @description Exact paid amount in micro units. */
            amount_micro: components["schemas"]["MicroAmount"];
            /**
             * Format: date-time
             * @description Payout confirmation timestamp.
             */
            created_at: string;
            /**
             * Format: date
             * @description Reward date in America/New_York.
             */
            epoch_date: string;
            /** @description Exchange this payout occurred on. */
            exchange: components["schemas"]["Exchange"];
            /** @description Namespaced payout item id. */
            id: components["schemas"]["PortfolioActivityId"];
            /** @description Canonical confirmed transaction hash, when available. */
            tx_hash?: string | null;
        };
        /** @description Complete-set merge activity. */
        PortfolioMergeActivity: {
            /** @description Collateral released. */
            amount_micro: components["schemas"]["MicroAmount"];
            /** @description Market condition id. */
            condition_id: components["schemas"]["ConditionId"];
            /**
             * Format: date-time
             * @description Ledger creation timestamp.
             */
            created_at: string;
            /** @description Exchange this activity occurred on. */
            exchange: components["schemas"]["Exchange"];
            /** @description Namespaced activity id. */
            id: components["schemas"]["PortfolioActivityId"];
            /** @description Shares burned per outcome leg. */
            shares_micro: components["schemas"]["MicroAmount"];
            /** @description Settlement transaction hash. */
            tx_hash: string;
        };
        /**
         * @description Open-orders list request body.
         * @example {
         *       "limit": 100,
         *       "token_ids": [
         *         "21742633143463906290569050155826241533067272736897614950488156847949938836455"
         *       ]
         *     }
         */
        PortfolioOpenOrdersListRequest: {
            /**
             * @description Opaque cursor from a prior response's `next_cursor`; omit for the first
             *     page.
             */
            cursor?: string | null;
            /** @description Optional AGARA filter. Omit it to query the AGARA wallet. */
            exchanges?: components["schemas"]["Exchange"][];
            /** @description Page limit. */
            limit?: components["schemas"]["PortfolioListLimit"] | null;
            /** @description Token ids to include. */
            token_ids?: components["schemas"]["TokenId"][];
        };
        /** @description Open-orders response body. */
        PortfolioOpenOrdersResponse: {
            /**
             * Format: date-time
             * @description Snapshot timestamp.
             */
            as_of: string;
            /** @description Event display metadata keyed by event slug. */
            events: {
                [key: string]: components["schemas"]["EventMeta"];
            };
            /** @description Market display metadata keyed by outcome token id. */
            markets: {
                [key: string]: components["schemas"]["MarketMeta"];
            };
            /** @description Open order rows. */
            orders: components["schemas"]["Order"][];
            /** @description Keyset pagination state. */
            pagination: components["schemas"]["CursorPagination_PortfolioListLimit"];
        };
        /** @description Order activity aggregated across its recorded fills. */
        PortfolioOrderActivity: {
            /** @description Filled-share weighted average price. */
            average_fill_price_micro: components["schemas"]["MicroAmount"];
            /** @description Market condition id when chain metadata is available. */
            condition_id?: components["schemas"]["ConditionId"] | null;
            /**
             * Format: date-time
             * @description Earliest recorded fill timestamp.
             */
            created_at: string;
            /** @description Exchange this activity occurred on. */
            exchange: components["schemas"]["Exchange"];
            /** @description Total fees charged across fills. */
            fees_micro: components["schemas"]["MicroAmount"];
            /** @description Whether the order filled completely or partially. */
            fill_status: components["schemas"]["PortfolioOrderFillStatus"];
            /** @description Total filled notional. */
            filled_amount_micro: components["schemas"]["MicroAmount"];
            /** @description Total filled shares. */
            filled_shares_micro: components["schemas"]["MicroAmount"];
            /** @description Activity id. */
            id: components["schemas"]["PortfolioActivityId"];
            /** @description Order id. */
            order_id: components["schemas"]["OrderId"];
            /** @description Order side. */
            side: components["schemas"]["Side"];
            /** @description Current order status. */
            status: components["schemas"]["ClobOrderStatus"];
            /** @description Outcome token id when chain metadata is available. */
            token_id?: components["schemas"]["TokenId"] | null;
        };
        /**
         * @description Order fill status.
         * @enum {string}
         */
        PortfolioOrderFillStatus: "FULL" | "PARTIAL";
        /** @description Position row. */
        PortfolioPosition: {
            /** @description Shares available to place into a new sell order. */
            available_shares_micro: components["schemas"]["MicroAmount"];
            /** @description Average entry price. */
            avg_price_micro: components["schemas"]["MicroAmount"];
            /** @description Market condition id. */
            condition_id: components["schemas"]["ConditionId"];
            /** @description Current price. */
            current_price_micro: components["schemas"]["MicroAmount"];
            /** @description Current position value. */
            current_value_micro: components["schemas"]["MicroAmount"];
            /** @description Owning exchange; token ids can overlap across listings. */
            exchange: components["schemas"]["Exchange"];
            /** @description Mergeable now: an active binary market with a held, locked-adjusted complete set. Advisory; engine is authority. */
            mergeable: boolean;
            /** @description Mergeable complete-set size (min available leg), micro. `None` when not mergeable; always `None` off AGARA. */
            mergeable_shares_micro?: components["schemas"]["MicroAmount"] | null;
            /** @description Profit or loss. */
            profit_loss_micro: components["schemas"]["MicroAmount"];
            /** @description Profit or loss ratio. */
            profit_loss_percent: components["schemas"]["PortfolioDecimalString"];
            /** @description Whether the position is redeemable. */
            redeemable: boolean;
            /** @description Held shares. */
            shares_micro: components["schemas"]["MicroAmount"];
            /** @description Maximum payout if the outcome wins. */
            to_win_micro: components["schemas"]["MicroAmount"];
            /** @description Outcome token id. */
            token_id: components["schemas"]["TokenId"];
        };
        /**
         * @description Request to merge complete outcome token sets into collateral.
         * @example {
         *       "condition_id": "0x21742633143463906290569050155826241533067272736897614950488156847949938836455",
         *       "shares_micro": "1000000"
         *     }
         */
        PortfolioPositionMergeRequest: {
            /** @description Market condition id. */
            condition_id: components["schemas"]["ConditionId"];
            /** @description Share amount in micro units. */
            shares_micro: components["schemas"]["MicroAmount"];
        };
        /**
         * @description Request to split collateral into outcome tokens.
         * @example {
         *       "collateral_amount_micro": "1000000",
         *       "condition_id": "0x21742633143463906290569050155826241533067272736897614950488156847949938836455"
         *     }
         */
        PortfolioPositionSplitRequest: {
            /** @description Collateral amount in micro units. */
            collateral_amount_micro: components["schemas"]["MicroAmount"];
            /** @description Market condition id. */
            condition_id: components["schemas"]["ConditionId"];
        };
        /**
         * @description Positions list request body. Positions are returned in one shot, so
         *     there are no page parameters.
         * @example {
         *       "condition_ids": [
         *         "0x21742633143463906290569050155826241533067272736897614950488156847949938836455"
         *       ]
         *     }
         */
        PortfolioPositionsListRequest: {
            /** @description Condition ids to include. */
            condition_ids: components["schemas"]["ConditionId"][];
            /** @description Optional AGARA filter. Omit it to query the AGARA wallet. */
            exchanges?: components["schemas"]["Exchange"][];
        };
        /**
         * @description Positions response body. Positions are returned in one unpaginated response.
         *     Treat the response as complete only when `unavailable_exchanges` is empty.
         */
        PortfolioPositionsResponse: {
            /**
             * Format: date-time
             * @description Snapshot timestamp.
             */
            as_of: string;
            /** @description Event display metadata keyed by event slug. */
            events: {
                [key: string]: components["schemas"]["EventMeta"];
            };
            /** @description Market display metadata keyed by outcome token id. */
            markets: {
                [key: string]: components["schemas"]["MarketMeta"];
            };
            /** @description Position rows. */
            positions: components["schemas"]["PortfolioPosition"][];
            /** @description Contains each exchange whose positions could not be fetched; those rows are absent. */
            unavailable_exchanges: components["schemas"]["Exchange"][];
        };
        /** @description Resolved outcome redemption activity. */
        PortfolioRedeemActivity: {
            /** @description Collateral received. */
            amount_micro: components["schemas"]["MicroAmount"];
            /** @description Market condition id. */
            condition_id: components["schemas"]["ConditionId"];
            /**
             * Format: date-time
             * @description Ledger creation timestamp.
             */
            created_at: string;
            /** @description Exchange this activity occurred on. */
            exchange: components["schemas"]["Exchange"];
            /** @description Namespaced activity id. */
            id: components["schemas"]["PortfolioActivityId"];
            /** @description Winning outcome shares redeemed. */
            shares_micro: components["schemas"]["MicroAmount"];
            /** @description Winning outcome token id. */
            token_id: components["schemas"]["TokenId"];
            /** @description Settlement transaction hash. */
            tx_hash: string;
        };
        /** @description Complete-set split activity. */
        PortfolioSplitActivity: {
            /** @description Collateral consumed. */
            amount_micro: components["schemas"]["MicroAmount"];
            /** @description Market condition id. */
            condition_id: components["schemas"]["ConditionId"];
            /**
             * Format: date-time
             * @description Ledger creation timestamp.
             */
            created_at: string;
            /** @description Exchange this activity occurred on. */
            exchange: components["schemas"]["Exchange"];
            /** @description Namespaced activity id. */
            id: components["schemas"]["PortfolioActivityId"];
            /** @description Shares minted per outcome leg. */
            shares_micro: components["schemas"]["MicroAmount"];
            /** @description Settlement transaction hash. */
            tx_hash: string;
        };
        /**
         * @description One exchange's portfolio summary. Collateral and positions on different
         *     networks remain separate.
         */
        PortfolioSummaryEntry: {
            /**
             * Format: date-time
             * @description Snapshot timestamp.
             */
            as_of: string;
            /** @description Wallet collateral balance. */
            cash_balance_micro: components["schemas"]["MicroAmount"];
            /** @description Exchange this entry summarises. */
            exchange: components["schemas"]["Exchange"];
            /** @description Cash available for buy validation. */
            free_cash_micro: components["schemas"]["MicroAmount"];
            /** @description Cost basis of open positions. */
            open_cost_basis_micro: components["schemas"]["MicroAmount"];
            /** @description Unrealized P&L for open positions. */
            open_unrealized_pnl_micro: components["schemas"]["MicroAmount"];
            /** @description Total portfolio value. */
            portfolio_value_micro: components["schemas"]["MicroAmount"];
            /** @description Current value of open positions. */
            positions_value_micro: components["schemas"]["MicroAmount"];
        };
        /** @description AGARA portfolio summary. */
        PortfolioSummaryResponse: {
            /** @description AGARA portfolio summary. */
            summaries: components["schemas"]["PortfolioSummaryEntry"][];
        };
        /** @description Trades response body. */
        PortfolioTradesResponse: {
            /**
             * Format: date-time
             * @description Snapshot timestamp.
             */
            as_of: string;
            /** @description Event display metadata keyed by event slug. */
            events: {
                [key: string]: components["schemas"]["EventMeta"];
            };
            /** @description Market display metadata keyed by outcome token id. */
            markets: {
                [key: string]: components["schemas"]["MarketMeta"];
            };
            /** @description Keyset pagination state. */
            pagination: components["schemas"]["CursorPagination_PortfolioListLimit"];
            /** @description Trade rows. */
            trades: components["schemas"]["Fill"][];
            /** @description Contains each exchange whose trades could not be fetched; those rows are absent. */
            unavailable_exchanges: components["schemas"]["Exchange"][];
        };
        /** @description Completed withdrawal activity. */
        PortfolioWithdrawalActivity: {
            /** @description Positive magnitude; the variant conveys direction. */
            amount_micro: components["schemas"]["MicroAmount"];
            /**
             * Format: int64
             * @description Chain the transfer settled on.
             */
            chain_id: number | bigint;
            /** @description On-chain transfer peer. */
            counterparty_address: string;
            /**
             * Format: date-time
             * @description Ledger creation timestamp.
             */
            created_at: string;
            /** @description Exchange this activity occurred on. */
            exchange: components["schemas"]["Exchange"];
            /** @description Namespaced activity id. */
            id: components["schemas"]["PortfolioActivityId"];
            /** @description Lowercased collateral token address. */
            token_address: string;
            /** @description Settlement transaction hash. */
            tx_hash: string;
        };
        PositionOperationAccepted: {
            /** Format: date-time */
            as_of: string;
            batch_hash: string;
            /** @enum {string} */
            status: "PENDING";
        };
        /** @description One exact UTC bucket. */
        RealizedPnlBucketDto: {
            /** @description Complete-set merge PnL. */
            directMerges: string;
            /** @description Resolution redemption PnL. */
            redemptions: string;
            /** @description Ordinary and cross-order sale PnL. */
            sales: string;
            /**
             * Format: date-time
             * @description Inclusive RFC 3339 UTC bucket start.
             */
            timestamp: string;
            /** @description Sum of all realised-PnL categories. */
            total: string;
        };
        /**
         * @description UTC resolution accepted by the realised-PnL endpoint.
         * @enum {string}
         */
        RealizedPnlBucketGranularityDto: "hour" | "day" | "week";
        /** @description Exact category totals for one UTC window. */
        RealizedPnlCategoryTotalsDto: {
            /** @description Complete-set merge PnL. */
            directMerges: string;
            /** @description Resolution redemption PnL. */
            redemptions: string;
            /** @description Ordinary and cross-order sale PnL. */
            sales: string;
            /** @description Sum of all realised-PnL categories. */
            total: string;
        };
        /** @description Complete exact realised-PnL report. */
        RealizedPnlReportDto: {
            /**
             * Format: int32
             * @description Decimal places in every amount string.
             */
            amountScale: number;
            /**
             * Format: date-time
             * @description Recognition time of the durable source prefix.
             */
            asOf: string;
            /** @description Ascending zero-filled UTC buckets for the requested window. */
            buckets: components["schemas"]["RealizedPnlBucketDto"][];
            /**
             * Format: date-time
             * @description Earliest instant proven by activation.
             */
            coverageStartedAt: string;
            /** @description Report currency. */
            currency: string;
            /** @description Whether the current UTC bucket can still receive disposals. */
            currentBucketPartial: boolean;
            /** @description Requested UTC bucket resolution. */
            granularity: components["schemas"]["RealizedPnlBucketGranularityDto"];
            /**
             * Format: int32
             * @description Recognised wallet operations not yet applied; zero does not prove the projector is current.
             */
            pendingOperations: number;
            /**
             * Format: date-time
             * @description Earliest disposal recognition instant included in totals and buckets.
             */
            reportingStartedAt: string;
            /** @description Last contiguous source event included. */
            throughEvent: components["schemas"]["RealizedPnlThroughEventDto"];
            /** @description Bucket timezone. */
            timezone: string;
            /** @description Seven-day, thirty-day, and all-time totals. */
            totals: components["schemas"]["RealizedPnlTotalsDto"];
            /** @description Requested UTC calendar window. */
            window: components["schemas"]["RealizedPnlWindowDto"];
        };
        /** @description Durable source frontier included by a report. */
        RealizedPnlThroughEventDto: {
            /** @description Exact event sequence as a decimal string. */
            eventSeq: string;
            /**
             * Format: int32
             * @description Engine shard.
             */
            shardId: number;
        };
        /** @description Exact realised-PnL totals. */
        RealizedPnlTotalsDto: {
            /** @description All realised PnL recognized at or after `reportingStartedAt` through `throughEvent`. */
            allTime: components["schemas"]["RealizedPnlCategoryTotalsDto"];
            /** @description Current UTC day plus the preceding twenty-nine days. */
            last30Days: components["schemas"]["RealizedPnlCategoryTotalsDto"];
            /** @description Current UTC day plus the preceding six days. */
            last7Days: components["schemas"]["RealizedPnlCategoryTotalsDto"];
        };
        /**
         * @description UTC calendar window accepted by the realised-PnL endpoint.
         * @enum {string}
         */
        RealizedPnlWindowDto: "1d" | "7d" | "30d" | "all";
        /** @description Accrued rebate balances across a user's wallets in micro-collateral units. */
        RebatesSummaryResponse: {
            /** @description Pending LP-incentive balance. */
            lp_incentive_micro: components["schemas"]["MicroAmount"];
            /** @description Pending maker-rebate balance. */
            maker_rebate_micro: components["schemas"]["MicroAmount"];
            /** @description Sum across all programs. */
            total_micro: components["schemas"]["MicroAmount"];
            /** @description Pending VIP taker-rebate balance. */
            vip_rebate_micro: components["schemas"]["MicroAmount"];
        };
        /** @description A durable resource referenced by check-status recovery. */
        RecoveryResource: {
            /** @enum {string} */
            kind: "order";
            /** @description Order UUID. */
            order_id: string;
        } | {
            /** @description Canonical 32-byte batch hash. */
            batch_hash: string;
            /** @enum {string} */
            kind: "batch";
        } | {
            /** @description Group UUID. */
            group_id: string;
            /** @enum {string} */
            kind: "group";
        } | {
            /** @enum {string} */
            kind: "wallet_status";
        };
        /**
         * @description Buy/sell side of a trade or order — the shared DTO side across the
         *     portfolio and orders endpoints.
         * @enum {string}
         */
        Side: "BUY" | "SELL";
        StatusResponse: {
            /** Format: int64 */
            events: number | bigint;
            /** Format: int64 */
            markets: number | bigint;
        };
        /** @description Outcome token identifier. */
        TokenId: string;
        /** @description Trade identifier. */
        TradeId: string;
        /** @description Portfolio trade status. */
        TradeStatus: string;
    };
    responses: never;
    parameters: never;
    requestBodies: never;
    headers: never;
    pathItems: never;
}
export type $defs = Record<string, never>;
export interface operations {
    status: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Public health/status snapshot: counts of markets and events known to the platform. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["StatusResponse"];
                };
            };
            /** @description A supplied access token is invalid or expired. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description A supplied identity token could not be verified because the identity provider is temporarily unavailable. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    create_order: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["CreateClobOrderRequest"];
            };
        };
        responses: {
            /** @description Order accepted; engine match lands asynchronously. */
            202: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["CreateClobOrderResponse"];
                };
            };
            /** @description The request body is not valid JSON. */
            400: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `orders:place` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Caller has no registered wallet. */
            404: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Request body exceeds the server limit; send a smaller payload. */
            413: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or unsupported `Content-Type`; send request bodies as `application/json`. */
            415: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The JSON body does not match the schema, the order fields fail validation, or the order was rejected (insufficient balance or shares, FOK unfillable, post-only would cross, market halted). */
            422: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Wallet setup or signer authorization is incomplete. */
            424: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The order provider rejected router credentials or returned an invalid response. */
            502: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The identity or order provider is temporarily unavailable. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    create_signed_order: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["CreateSignedClobOrderRequest"];
            };
        };
        responses: {
            /** @description Pre-signed order accepted; submission ack via the engine event stream. */
            202: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["CreateClobOrderResponse"];
                };
            };
            /** @description The request body is not valid JSON. */
            400: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `orders:place_signed` scope, or trading is disabled for the account. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Caller has no registered wallet. */
            404: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description An order with the same hash already exists. */
            409: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Request body exceeds the server limit; send a smaller payload. */
            413: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or unsupported `Content-Type`; send request bodies as `application/json`. */
            415: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The JSON body does not match the schema, order fields fail validation, token ids disagree, or the hash or signature is invalid. */
            422: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Wallet setup or signer authorization is incomplete. */
            424: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The order provider rejected router credentials or returned an invalid response. */
            502: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The identity or order provider is temporarily unavailable. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    create_signed_order_batch: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["CreateSignedClobOrderBatchRequest"];
            };
        };
        responses: {
            /** @description Batch processed; `results` carries one outcome per submitted order in request order. Expected per-order rejections contain canonical public failure metadata. */
            202: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["CreateSignedClobOrderBatchResponse"];
                };
            };
            /** @description The request body is not valid JSON. */
            400: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `orders:place_signed` scope, or trading is disabled for the account. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Caller has no registered wallet. */
            404: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Request body exceeds the server limit; send a smaller payload. */
            413: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or unsupported `Content-Type`; send request bodies as `application/json`. */
            415: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The JSON body does not match the schema, the batch is empty, or it carries more than 32 orders. */
            422: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Wallet setup or signer authorization is incomplete. */
            424: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The order provider rejected router credentials or returned an invalid response. */
            502: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The identity or order provider is temporarily unavailable. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    list_orders: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["ClobOrdersListRequest"];
            };
        };
        responses: {
            /** @description Your orders, newest first. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ClobOrdersListResponse"];
                };
            };
            /** @description The request body is not valid JSON. */
            400: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `orders:read` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Caller has no registered wallet. */
            404: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Request body exceeds the server limit; send a smaller payload. */
            413: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or unsupported `Content-Type`; send request bodies as `application/json`. */
            415: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The JSON body is well-formed but does not match the schema — a required field is missing, or a field has the wrong type. */
            422: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Identity verification is temporarily unavailable. Retry with backoff. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    get_order: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description Internal order UUID returned by `POST /trade/v1/orders`. */
                order_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Order detail with current status and fill state. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ClobOrderResponse"];
                };
            };
            /** @description The order_id path parameter is not a UUID. */
            400: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `orders:read` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description No order with this id, or not yours. */
            404: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Wallet setup is incomplete. */
            424: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Identity verification is temporarily unavailable. Retry with backoff. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    cancel_order: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description Internal order UUID returned by `POST /trade/v1/orders`. */
                order_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Cancellation request accepted; this is a no-op when the order is already cancelled. */
            202: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["CancelClobOrderResponse"];
                };
            };
            /** @description The order_id path parameter is not a UUID. */
            400: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `orders:cancel` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description No order with this id, or not yours. */
            404: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description A pre-venue order was already terminal other than CANCELLED, or raced to another terminal state before cancellation was recorded. */
            409: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Wallet setup or signer authorization is incomplete. */
            424: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The order provider rejected router credentials or returned an invalid response. */
            502: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The identity or order provider is temporarily unavailable. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    get_order_trades: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description Internal order UUID returned by `POST /trade/v1/orders`. */
                order_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Trades that filled this order, newest-first, from the order's perspective. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["OrderTradesResponse"];
                };
            };
            /** @description The order_id path parameter is not a UUID. */
            400: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `orders:read` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description No order with this id, or not yours. */
            404: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Wallet setup is incomplete. */
            424: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Identity verification is temporarily unavailable. Retry with backoff. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    cancel_all_orders: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Per-wallet cancellation sweeps were scheduled for current nonterminal orders. */
            202: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["CancelAllClobOrdersResponse"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `orders:cancel_all` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description No Agara wallet is registered. */
            404: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Identity verification is temporarily unavailable. Retry with backoff. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    create_batch: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["AccountBatchSubmission"];
            };
        };
        responses: {
            /** @description Batch accepted; durable execution started. */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["AccountBatchAccepted"];
                };
            };
            /** @description The request body is not valid JSON. */
            400: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `batches:submit` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Another batch is already in flight, or the submitted sequence conflicts with the wallet's current state. Problem Details `code` identifies the condition. */
            409: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Request body exceeds the server limit; send a smaller payload. */
            413: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or unsupported `Content-Type`; send request bodies as `application/json`. */
            415: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The JSON body does not match the schema, or the batch violates a typed signature, market, balance, value, or route constraint. Problem Details `code` identifies the condition. */
            422: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Wallet setup or signer authorization is incomplete. */
            424: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The relayer rejected router credentials, returned an invalid response, or left workflow-start status uncertain. */
            502: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Account batches are not configured, or the engine or relayer is temporarily unavailable. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    get_batch: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description EIP-712 batch digest, 0x hex */
                batch_hash: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description The batch lifecycle row. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["AccountBatchStatusDto"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `batches:submit` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unknown hash, or not this wallet's batch. */
            404: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Account batches are not configured. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    supersede_batch: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description The PENDING batch to replace */
                batch_hash: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["AccountBatchSupersedeSubmission"];
            };
        };
        responses: {
            /** @description The batch already left PENDING; body carries its state. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["AccountBatchSupersedeOutcome"];
                };
            };
            /** @description Successor accepted at the same seq. */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["AccountBatchSupersedeOutcome"];
                };
            };
            /** @description The request body is not valid JSON. */
            400: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `batches:submit` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unknown hash, or not this wallet's batch. */
            404: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The successor conflicts with another batch in flight. */
            409: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Request body exceeds the server limit; send a smaller payload. */
            413: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or unsupported `Content-Type`; send request bodies as `application/json`. */
            415: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The JSON body does not match the schema, or the successor batch is invalid. */
            422: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Wallet setup or signer authorization is incomplete. */
            424: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The relayer rejected router credentials, returned an invalid response, or left workflow-start status uncertain. */
            502: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Account batches are not configured, or the engine or relayer is temporarily unavailable. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    agara_orderbook: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                /** @description Outcome token id (decimal-encoded uint256). */
                token_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Live bid/ask depth. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["AgaraOrderbookResponse"];
                };
            };
            /** @description A supplied access token is invalid or expired. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token not found or not served by this endpoint. */
            404: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The market-data dependency returned an invalid response. */
            502: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description A supplied identity token could not be verified because the identity provider is temporarily unavailable. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    portfolio_activities: {
        parameters: {
            query?: {
                /** @description Page size from 1 to 500. Defaults to 50. */
                limit?: number;
                /** @description Opaque `next_cursor` from the previous page. */
                cursor?: string;
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Current AGARA history of orders with fills, splits, merges, redemptions, deposits, withdrawals, and confirmed LP payouts, sorted newest-first. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PortfolioActivitiesResponse"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `portfolio:read` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The limit or cursor is invalid. */
            422: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Identity verification is temporarily unavailable. Retry with backoff. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    portfolio_bridge_withdraw_supported_assets: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Chains and assets supported for withdrawing from your trading wallet. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PortfolioBridgeSupportedAssetsResponse"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `portfolio:read` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Identity verification is temporarily unavailable. Retry with backoff. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    portfolio_bridge_withdraw_quote: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["PortfolioBridgeWithdrawQuoteRequest"];
            };
        };
        responses: {
            /** @description Price + estimated time for a cross-chain withdrawal. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PortfolioBridgeDepositQuoteResponse"];
                };
            };
            /** @description The request body is not valid JSON. */
            400: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `portfolio:read` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Caller has no AGARA wallet. */
            404: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Request body exceeds the server limit; send a smaller payload. */
            413: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or unsupported `Content-Type`; send request bodies as `application/json`. */
            415: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The JSON body does not match the schema, or the asset, chain, or amount is unsupported. */
            422: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description AGARA wallet setup is incomplete. */
            424: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The bridge provider rejected router credentials or returned an invalid response. */
            502: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The identity or bridge provider is temporarily unavailable. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    portfolio_open_orders_list: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["PortfolioOpenOrdersListRequest"];
            };
        };
        responses: {
            /** @description Active nonterminal orders across all your wallets. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PortfolioOpenOrdersResponse"];
                };
            };
            /** @description The request body is not valid JSON. */
            400: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `portfolio:read` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Request body exceeds the server limit; send a smaller payload. */
            413: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or unsupported `Content-Type`; send request bodies as `application/json`. */
            415: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The JSON body is well-formed but does not match the schema — a required field is missing, or a field has the wrong type. */
            422: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Identity verification is temporarily unavailable. Retry with backoff. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    portfolio_positions_list: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["PortfolioPositionsListRequest"];
            };
        };
        responses: {
            /** @description Your current positions, with optional filter by condition_id. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PortfolioPositionsResponse"];
                };
            };
            /** @description The request body is not valid JSON. */
            400: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `portfolio:read` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Request body exceeds the server limit; send a smaller payload. */
            413: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or unsupported `Content-Type`; send request bodies as `application/json`. */
            415: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The JSON body is well-formed but does not match the schema — a required field is missing, or a field has the wrong type. */
            422: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Identity verification is temporarily unavailable. Retry with backoff. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    portfolio_positions_split: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["PortfolioPositionSplitRequest"];
            };
        };
        responses: {
            /** @description AGARA: split accepted; poll GET /trade/v1/batches/{batch_hash} for settlement. */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PositionOperationAccepted"];
                };
            };
            /** @description The request body is not valid JSON. */
            400: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `positions:split` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Caller has no wallet for this market's exchange. */
            404: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description AGARA: the wallet's batch queue is full, or a merge-all is settling; retry once one finishes. */
            409: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Request body exceeds the server limit; send a smaller payload. */
            413: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or unsupported `Content-Type`; send request bodies as `application/json`. */
            415: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The JSON body does not match the schema, or the AGARA account batch was refused because of coverage, value, or route constraints. */
            422: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description AGARA: wallet setup incomplete (no embedded wallet or signer authorization). */
            424: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The provider or relayer rejected router credentials or returned an invalid response. */
            502: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Position batches are not configured, or the provider, engine, or relayer is temporarily unavailable. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    portfolio_positions_merge: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["PortfolioPositionMergeRequest"];
            };
        };
        responses: {
            /** @description AGARA: merge accepted; poll GET /trade/v1/batches/{batch_hash} for settlement. */
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PositionOperationAccepted"];
                };
            };
            /** @description The request body is not valid JSON. */
            400: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `positions:merge` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Caller has no wallet for this market's exchange. */
            404: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description AGARA: the wallet's batch queue is full, or a merge-all is settling; retry once one finishes. */
            409: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Request body exceeds the server limit; send a smaller payload. */
            413: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing or unsupported `Content-Type`; send request bodies as `application/json`. */
            415: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The JSON body does not match the schema, or the AGARA account batch was refused because of coverage, value, or route constraints. */
            422: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description AGARA: wallet setup incomplete (no embedded wallet or signer authorization). */
            424: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The provider or relayer rejected router credentials or returned an invalid response. */
            502: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Position batches are not configured, or the provider, engine, or relayer is temporarily unavailable. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    portfolio_summary: {
        parameters: {
            query?: {
                /** @description Optional AGARA exchange filter. Omit it to query the AGARA wallet. */
                exchanges?: string;
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Aggregated collateral balance and open-position valuation across all your wallets. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PortfolioSummaryResponse"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `portfolio:read` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The query or `exchanges` filter is invalid. */
            422: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Identity verification is temporarily unavailable. Retry with backoff. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    portfolio_trades: {
        parameters: {
            query?: {
                /** @description Page size. Default 500, max 500. */
                limit?: number;
                /** @description Opaque cursor from a prior response's `next_cursor`; omit for the first page. */
                cursor?: string;
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Fill history sorted newest-first, keyset-paginated. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["PortfolioTradesResponse"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `portfolio:read` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The limit or cursor query parameter is invalid. */
            422: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Identity verification is temporarily unavailable. Retry with backoff. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    portfolio_rebates: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Pending rebate balances per program, in micro-collateral units. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["RebatesSummaryResponse"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `portfolio:read` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Identity verification is temporarily unavailable. Retry with backoff. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    lp_incentives: {
        parameters: {
            query?: {
                /** @description Active root-category slug. */
                category?: string;
                /** @description Case-insensitive literal substring matched against market question and event title. */
                search?: string;
                /** @description Market field used for explicit sorting. */
                sort_by?: components["schemas"]["LpIncentiveSortBy"];
                /** @description Sort direction. Defaults to `desc` when `sort_by` is supplied. */
                sort_order?: components["schemas"]["LpIncentiveSortOrder"];
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Active and upcoming LP-incentive opportunities. Anonymous callers get the same markets with zeroed personal standing. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["LpIncentivesResponse"];
                };
            };
            /** @description Invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `portfolio:read` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Category, search, or sorting parameters are invalid. */
            422: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Identity verification is temporarily unavailable. Retry with backoff. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    lp_incentive_categories: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Active root categories with distinct current incentive-market counts greater than zero. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["LpIncentiveCategoriesResponse"];
                };
            };
            /** @description Supplied authentication credentials are invalid or expired. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Identity verification is temporarily unavailable. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    closed_lp_incentives: {
        parameters: {
            query?: {
                /** @description Page size from 1 to 100. Defaults to 20. */
                limit?: number;
                /** @description 1-based page number. Defaults to 1. */
                page?: number;
                /** @description Active root-category slug. */
                category?: string;
                /** @description Case-insensitive literal substring matched against market question and event title. */
                search?: string;
                /** @description Row field used for explicit sorting. Defaults to `date` (the credited reward day). */
                sort_by?: components["schemas"]["ClosedLpIncentiveSortBy"];
                /** @description Sort direction. Defaults to `desc` when `sort_by` is supplied. */
                sort_order?: components["schemas"]["LpIncentiveSortOrder"];
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description One page of your credited LP reward cycles — one row per accrual, using the accrual's reward day — with the unpaged row count. Callers with no rewards get an empty list. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ClosedLpIncentivesResponse"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `portfolio:read` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The authenticated identity has not completed registration. */
            404: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Limit, page, category, search, or sorting parameters are invalid. */
            422: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Identity verification is temporarily unavailable. Retry with backoff. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    closed_lp_incentive_categories: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Active root categories containing markets you earned LP rewards in, with per-category market counts. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["LpIncentiveCategoriesResponse"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `portfolio:read` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The authenticated identity has not completed registration. */
            404: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Identity verification is temporarily unavailable. Retry with backoff. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    lp_incentive_earnings: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Rewards for the current reward cycle so far — projected live from today's scores — plus the trailing 7-day, 30-day, and all-time windows ending today. Callers with no scores or accruals get zeros. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["LpIncentiveEarningsResponse"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `portfolio:read` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The authenticated identity has not completed registration. */
            404: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Unexpected server-side failure. Do not retry automatically; retain the request ID and contact support. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Identity verification is temporarily unavailable. Retry with backoff. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
    portfolio_realized_pnl: {
        parameters: {
            query: {
                /** @description Required UTC bucket resolution. Use `hour` or `day` with fixed windows, and `week` with `all`. */
                granularity: components["schemas"]["RealizedPnlBucketGranularityDto"];
                /** @description Required UTC calendar window. Use `1d`, `7d`, or `30d` with hour/day buckets, and `all` with week buckets. */
                window: components["schemas"]["RealizedPnlWindowDto"];
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Timestamped UTC buckets for the requested calendar window, plus exact seven-day, thirty-day, and all-time totals since reporting started. */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["RealizedPnlReportDto"];
                };
            };
            /** @description Missing or invalid access token. */
            401: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Token lacks the `portfolio:read` scope. */
            403: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description No AGARA wallet is registered. */
            404: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Missing, unsupported, or incompatible query parameters. */
            422: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The durable projection has not caught up. */
            425: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description Rate limited. `Retry-After` carries the seconds to wait. When present, `X-RateLimit-*` describes the per-user bucket evaluated for the request; handler-level, limiter-unavailable, and edge-generated responses may omit it. The request was not processed, so the identical request is safe to resend after backing off. */
            429: {
                headers: {
                    /** @description Whole seconds to wait before retrying the identical request. */
                    "Retry-After"?: number;
                    /** @description Per-user bucket evaluated for the request. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Bucket"?: "read" | "place" | "place_batch" | "cancel" | "cancel_all";
                    /** @description Evaluated per-user bucket capacity. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Limit"?: number | bigint;
                    /** @description Tightest remaining per-user budget. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Remaining"?: number | bigint;
                    /** @description Whole seconds until the evaluated per-user bucket is full. Handler-level, limiter-unavailable, and edge-generated 429 responses may omit it. */
                    "X-RateLimit-Reset"?: number | bigint;
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["KnownEdgeProblem"];
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The projection violated an internal invariant. */
            500: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
            /** @description The projection failed closed or its repository is unavailable. */
            503: {
                headers: {
                    /** @description Origin request identifier to retain when contacting support. Edge-generated 429 responses may omit it. */
                    "X-Request-ID"?: string;
                    [name: string]: unknown;
                };
                content: {
                    "application/problem+json": components["schemas"]["KnownOriginProblemDetails"];
                };
            };
        };
    };
}
