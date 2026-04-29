export function evaluatePolicyDelta({ previousScore, currentScore }) {
  const measuredDelta = Number((currentScore - previousScore).toFixed(4));
  return {
    measured_delta: measuredDelta,
    regression: measuredDelta < 0,
  };
}
