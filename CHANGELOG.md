# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.7.0 (2024-09-16)

### New Features

 - <csr-id-dc2ff98c787ad9d4150f2a89737cd686a555de09/> Add `From<&Chapter>` implementation to ChapterListElem
   ChapterListElem is in reality a tuple (usize, String).
   Also add a new variant to BackendError

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Add `From<&Chapter>` implementation to ChapterListElem (dc2ff98)
</details>

## v0.6.0 (2024-09-16)

<csr-id-a6b39541556546f006421073cb045675236c0c78/>

### Chore

 - <csr-id-a6b39541556546f006421073cb045675236c0c78/> Update README.md

### New Features

 - <csr-id-0336e08a1a27ba814303f2d79673a43709007bd6/> Add `get_chapter_list` to backends
   This functions returns a `(index: usize, title: String)` tuple that can be used to compare chapters without making an individual request to each chapter. This is mainly useful to detect deletions on the source versus a local cache.
 - <csr-id-014de273835b5b3164f623f4fabc9b71224660ab/> Refactor freewebnovel selectors to use LazyLock for minor efficiency improvements.
 - <csr-id-5084ee58a871348abdd6d4ef04343ecea488c190/> Refactor RR's selectors for a minor efficiency improvement.
   Less used CPU cycles are always better, right?
 - <csr-id-6b84dd207182148e610aeb2c5d9f8a7689b984cd/> Build RR's regex list only once
   This results in a very small, as in barely measurable, performance improvement.

### Bug Fixes

 - <csr-id-2302f81ee5a605eba06847c2bf98c17184c9f5d9/> don't assume RR's antitheft message have a <p> class.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release libwebnovel v0.6.0 (055223a)
    - Update README.md (a6b3954)
    - Add `get_chapter_list` to backends (0336e08)
    - Refactor freewebnovel selectors to use LazyLock for minor efficiency improvements. (014de27)
    - Refactor RR's selectors for a minor efficiency improvement. (5084ee5)
    - Don't assume RR's antitheft message have a <p> class. (2302f81)
    - Build RR's regex list only once (6b84dd2)
</details>

## v0.5.0 (2024-09-15)

<csr-id-54fc1303eca5b44793583949e1c0ddb5a11730ec/>

### Chore

 - <csr-id-54fc1303eca5b44793583949e1c0ddb5a11730ec/> bump version

### New Features

 - <csr-id-aad742d4c9f9bc70e845ee11e89e36605c645569/> Add immutable_identifier method to backends.
   This method returns something that won't change if the title of the novel changes.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release libwebnovel v0.5.0 (280f59b)
    - Bump version (54fc130)
    - Add immutable_identifier method to backends. (aad742d)
</details>

## v0.4.1 (2024-09-15)

<csr-id-b1c257fe7bb9c2ac312cb9dc0c40f9265581703c/>
<csr-id-a337c3a06c31f170c165b408fffc8c3d81a980bf/>
<csr-id-adccec54e668ac09231251110ff48f3a9eb2f0fa/>
<csr-id-9e4eba2103d4142e33c64fa51e61a0cbb2bc2b7e/>
<csr-id-5f43f7ad5119daf37169dc02d4608b755bbdf108/>

### Chore

 - <csr-id-b1c257fe7bb9c2ac312cb9dc0c40f9265581703c/> Exclude CHANGELOG from eof pre-commit hook.
   Yeah, there's a trailing space on changelog generation, which blocks the release.
 - <csr-id-a337c3a06c31f170c165b408fffc8c3d81a980bf/> update locked deps
 - <csr-id-adccec54e668ac09231251110ff48f3a9eb2f0fa/> bump version
 - <csr-id-9e4eba2103d4142e33c64fa51e61a0cbb2bc2b7e/> Add configuration for [pre-commit](https://pre-commit.com) support
 - <csr-id-5f43f7ad5119daf37169dc02d4608b755bbdf108/> use map_err instead of or_else for RoyalRoad author parsing as per clippy suggestion

### New Features

 - <csr-id-5a76e30e2494778946d4928a7fa155f5a048d303/> refactor error types when parsing Chapter
 - <csr-id-67ed3c0c2b2b2c83c7e9abe9019288ac16219224/> Use only one HTTP client
   Use LazyLock for HTTP client to improve initialization. This reduces the overhead of creating a new client for each request by reusing a single instance, thus hopefully opening only one TCP connection. Maybe that will convince webnovel hosts that we are not a threat.

### Bug Fixes

 - <csr-id-6ce7fb90e4b138bf3c5268734212a891e163a5ca/> first draft of attempt to remove RR's added text when reading outside of their website.
   It makes sense to prevent reading on other websites that steal content and present it as their own, much less for a local copy of something.
 - <csr-id-504a0d2fe889e8a76159bb0e7d241dc0e55a9dd0/> Use usize instead of u32 for chapter count/indexes

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release.
 - 4 days passed between releases.
 - 9 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release libwebnovel v0.4.1 (0a15e3a)
    - Exclude CHANGELOG from eof pre-commit hook. (b1c257f)
    - Update locked deps (a337c3a)
    - Bump version (adccec5)
    - Refactor error types when parsing Chapter (5a76e30)
    - Use only one HTTP client (67ed3c0)
    - Add configuration for [pre-commit](https://pre-commit.com) support (9e4eba2)
    - First draft of attempt to remove RR's added text when reading outside of their website. (6ce7fb9)
    - Use map_err instead of or_else for RoyalRoad author parsing as per clippy suggestion (5f43f7a)
    - Use usize instead of u32 for chapter count/indexes (504a0d2)
</details>

## v0.4.0 (2024-09-11)

<csr-id-3e715a82a79415666c46698c07f8ece387b187d2/>
<csr-id-eeb20604f7c59ba9b6be2a89325234487690c131/>

### Chore

 - <csr-id-3e715a82a79415666c46698c07f8ece387b187d2/> Update README.md
 - <csr-id-eeb20604f7c59ba9b6be2a89325234487690c131/> bump version

### New Features

 - <csr-id-e9f73bdbada3e30a2826ac02046d8184343f1366/> Add fiction cover image fetching

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 3 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release libwebnovel v0.4.0 (d3090f1)
    - Update README.md (3e715a8)
    - Bump version (eeb2060)
    - Add fiction cover image fetching (e9f73bd)
</details>

## v0.3.2 (2024-09-07)

<csr-id-b8a6fcee6dc9d2fa9511f992ea9d7cea0c9e0ca4/>

### Chore

 - <csr-id-b8a6fcee6dc9d2fa9511f992ea9d7cea0c9e0ca4/> bump version

### New Features

 - <csr-id-72606b68da6a671f9578534a2555ec2fc9302fe3/> Add request error handling on freewebnovel::get_chapter
   We previously didn't check the success of the http GET request and were unable to return an appropriate error in case of a request failure.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release libwebnovel v0.3.2 (7592ad2)
    - Bump version (b8a6fce)
    - Add request error handling on freewebnovel::get_chapter (72606b6)
</details>

## v0.3.1 (2024-09-06)

<csr-id-9f05c4800e4c1d715ac58d9fa3ac4a2623cbf751/>
<csr-id-a930c6734c1b5c968445750b6ef4de9ef2f813c2/>

### Chore

 - <csr-id-9f05c4800e4c1d715ac58d9fa3ac4a2623cbf751/> Update README.md
 - <csr-id-a930c6734c1b5c968445750b6ef4de9ef2f813c2/> add keywords & categories for crates.io

### New Features

 - <csr-id-62fbb66233d4da4e22e7fe71dcfd8fd914550470/> Add Chapter (de)serialization
   This is done with FromStr/Display.

### Bug Fixes

 - <csr-id-2094cbcacbec658410c04b9e364f248fe046c48d/> add link to docs on the docs badge

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release libwebnovel v0.3.1 (0e70a1c)
    - Update README.md (9f05c48)
    - Add Chapter (de)serialization (62fbb66)
    - Add keywords & categories for crates.io (a930c67)
    - Add link to docs on the docs badge (2094cbc)
</details>

## v0.3.0 (2024-09-06)

<csr-id-b5870a4ebdae5107a3c148c6743c2b7b8601e5cb/>

### Bug Fixes

 - <csr-id-3ff0b259721588d05fdb060223c588dd2e106d2e/> fix typo in release script
 - <csr-id-7c7610dc64c025bb1a04d17f3842f31141184b45/> add a check to commit the readme file only if it has changed
 - <csr-id-cfc71d3d2e1b02573709b9f5b8e18f3ac8c6c137/> commit the readme after being generated by `cargo readme` in the release process
 - <csr-id-60485dcc4324bcbf94004705b5b40ab38fee1987/> fix license specification in Cargo.toml

### Chore

 - <csr-id-b5870a4ebdae5107a3c148c6743c2b7b8601e5cb/> Update README.md

### Documentation

 - <csr-id-16e3a0b5a0d8478ade2a3e2de43b9d850d052e51/> include all features on docs generation
 - <csr-id-d5c35c9611482385221bd21f9a5e759b133537ed/> add more lints & fix missing intra links

### New Features

 - <csr-id-e7a5a934be10e0b0be5e8cba767d5845a33bb07d/> Add chapter ordering with function provided by backends
   This enables multiple things:
   - ordering the chapters (obvy);

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release.
 - 8 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release libwebnovel v0.3.0 (af83166)
    - Fix typo in release script (3ff0b25)
    - Add a check to commit the readme file only if it has changed (7c7610d)
    - Commit the readme after being generated by `cargo readme` in the release process (cfc71d3)
    - Update README.md (b5870a4)
    - Fix license specification in Cargo.toml (60485dc)
    - Release libwebnovel v0.3.0 (8113759)
    - Add chapter ordering with function provided by backends (e7a5a93)
    - Include all features on docs generation (16e3a0b)
    - Add more lints & fix missing intra links (d5c35c9)
</details>

## v0.2.0 (2024-09-05)

<csr-id-51636dcfc7821047a13dc386d35ae3e9a93e2f39/>

### Chore

 - <csr-id-51636dcfc7821047a13dc386d35ae3e9a93e2f39/> bump version

### Documentation

 - <csr-id-12d1a5a111675bfd38d5fa50fbbffe87320c7e60/> cleanup

### New Features

 - <csr-id-9e8018df10bbb66b7ad74384dad7e4d9ce04ddd9/> add freewebnovel and fix libread
 - <csr-id-f0f960f271f978a2cbdda5563f3e864eb072a849/> add smart-release & git push to release script
 - <csr-id-7f68a5ad056b4f31c4d1c101d67319c4b89bb807/> add release script

### Bug Fixes

 - <csr-id-f9307d20708bac6356413ae64ae2b398fdfd170e/> fix default Backend::get_chapters() implementation

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 12 commits contributed to the release.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release libwebnovel v0.2.0 (4371836)
    - Add freewebnovel and fix libread (9e8018d)
    - Add smart-release & git push to release script (f0f960f)
    - Release libwebnovel v0.1.1 (eac14d7)
    - Add release script (7f68a5a)
    - Bump version (51636dc)
    - Fix default Backend::get_chapters() implementation (f9307d2)
    - Cleanup (12d1a5a)
    - Remove Storage-related stuff as this will be in a separate crate (7a17b5d)
    - Add some more docs, fix metadata (dc6be84)
    - Initial working version (0592e60)
    - Initial commit (10f6bdf)
</details>

## v0.1.1 (2024-09-05)

<csr-id-51636dcfc7821047a13dc386d35ae3e9a93e2f39/>

### Chore

 - <csr-id-51636dcfc7821047a13dc386d35ae3e9a93e2f39/> bump version

### Documentation

 - <csr-id-12d1a5a111675bfd38d5fa50fbbffe87320c7e60/> cleanup

### New Features

 - <csr-id-f4d612e6d4fd843fcdd17397a254e07e1eda1020/> add release script
 - <csr-id-7f68a5ad056b4f31c4d1c101d67319c4b89bb807/> add release script

### Bug Fixes

 - <csr-id-f9307d20708bac6356413ae64ae2b398fdfd170e/> fix default Backend::get_chapters() implementation

