campaign name

pilot: clean and standardize --pilot-demo output

files allowed to change

examples/pilot/run_pilot.sh
crates/service/tests/pilot_demo_surface_tests.rs

Claude prompt

Refine the `--pilot-demo` output to be:

- perfectly aligned (no uneven spacing)
- deterministic (no timestamps, no randomness)
- visually clean:

PostCAD Pipeline

CAD
↓
PostCAD routing
↓
compliance verification
↓
manufacturing dispatch
↓
audit receipt

Rules:
- exact wording must stay stable
- no extra logs or noise
- no colors or ANSI codes
- stdout only

Update surface test if needed to enforce:
- presence of "PostCAD Pipeline"
- exact arrow structure using "↓"

test command

cd ~/projects/postcad && bash examples/pilot/run_pilot.sh --pilot-demo | grep -E "PostCAD Pipeline|PostCAD routing|audit receipt"

commit message

pilot: standardize demo output formatting
