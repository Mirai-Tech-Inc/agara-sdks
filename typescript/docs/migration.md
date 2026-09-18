# Current contract choices

This is the first TypeScript package; these notes help applications porting from the
older Python/Rust convenience APIs or handwritten integrations.

- Pass exact micro strings for unsigned orders. LIMIT BUY accepts shares, not a
  collateral budget. MARKET BUY uses collateral; MARKET SELL uses shares.
- Keep response envelopes. Position unavailability, markets/events metadata, timestamps
  and pagination are part of the result; incomplete does not mean empty.
- Use `is_terminal` and `waitForOrder`; status-name sets misclassify current FAK outcomes.
- Read nested `failure` for order/batch rejections and structured Problem Details for
  HTTP errors. Generic status-only retry rules are unsafe for mutations.
- Wait for AGARA split/merge's batch; HTTP 201 is acceptance, not settlement.
- Subscribe to event-scoped lifecycle changes with event_id. Those updates have no
  engine sequence. New/unknown messages are not zero-filled into old message models.
- API discovery `source` casing is route-specific and reflected by generated queries:
  markets uses `agara`, events/search use `AGARA`.
- Native 64-bit JSON numbers may be bigint; explicitly format for display and use the
  SDK serializer through client calls. JSON.stringify alone cannot serialize bigint.
- `viem` is an optional peer and signing lives in a separate subpath. REST-only users
  do not need private-key or Ethereum dependencies.
