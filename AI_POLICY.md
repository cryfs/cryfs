# AI Use Policy

CryFS is security-critical software. People trust it to keep their data
confidential, so it must be authored and audited by humans who understand it.
This policy explains how AI tools are — and are not — used in this project.

## Principles

- **A human is responsible, not AI.** The maintainer is accountable for the
  security and correctness of all code in this repository. That responsibility
  is never delegated to a tool.
- **AI never makes autonomous decisions.** AI is used only as a collaborator —
  to brainstorm ideas and to type code more quickly. It does not decide what
  gets merged.
- **Everything is reviewed by a human.** No AI-written change is merged without
  being thoroughly read, understood, and verified by a human first. AI output is
  treated as a draft to be checked, not as a finished result.
- **Security-relevant code gets extra scrutiny.** Crypto, integrity, and other
  security-sensitive code is held to the highest review bar regardless of how it
  was drafted.

## History and disclosure

- CryFS up to version 1.0.3 was written entirely without AI.
- CryFS 2.0.0-alpha3 is hand-written as well and is feature complete. This means
  all important functional pieces of CryFS 2.0 have been written without AI.
  In commits after 2.0.0-alpha3 towards the 2.0 release, AI has helped polish
  the code, write and fix tests. AI has **not** made any security-relevant
  changes in 2.0.
- Going forward (versions past 2.0), AI may be used more heavily, including for
  feature development — always under the principles above.

## For contributors

- You may use AI tools to help write your contribution.
- **Disclose it.** If AI meaningfully helped produce a change, say so in the pull
  request.
- You are responsible for any code you submit. Understand it, test it, and be
  able to explain and maintain it as if you had written every line yourself.
  "The AI wrote it" is not an acceptable answer to a review question.
