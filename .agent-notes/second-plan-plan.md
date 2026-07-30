# Experiment: does the IK retargeting claim hold for a second body plan?

Hypothesis under test (bodies.md, DECIDED 2026-07-19): "Two-bone IK for limbs +
neck look-at is the retargeting glue that makes one clip serve every mutation of
a plan ... across differing proportions."

Steps:
1. Route the client's body rendering through the REGISTRY (vanilla_body_pack as
   the first pack) instead of compiled-in biped_plan()/biped_clips().
2. Per-character plan selection, identity default dc:body/biped, unregistered
   named plan => receipt refusal (RejectReason::UnknownBodyPlan).
3. Author dc:body/stout: same 11 joints, ~0.5x legs, ~1.6x arms, wider/deeper
   trunk, bigger head. Same three clips. Measure.

Measurement: a #[cfg(test)] probe in dc-client/src/body.rs that FKs each leg
through every sampled clip frame and reports sole-vs-ground excursion for both
plans, plus IK reach/clamp saturation.
