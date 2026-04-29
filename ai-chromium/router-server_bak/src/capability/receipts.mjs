export function makeReceipt({ turnId, seq, provider, capability, status, targetId = null, evidenceRefs = [] }) {
  return {
    schema: "ai_chromium.action_receipt.v1",
    turn_id: turnId,
    seq,
    provider,
    capability,
    status,
    target_id: targetId,
    evidence_refs: evidenceRefs,
    created_at: new Date().toISOString(),
  };
}

export function makeReceiptStore(turnId) {
  const receipts = [];
  let seq = 0;

  return {
    add(fields) {
      const receipt = makeReceipt({ turnId, seq: ++seq, ...fields });
      receipts.push(receipt);
      return receipt;
    },
    all() { return [...receipts]; },
  };
}
