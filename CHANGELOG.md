# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.3.0 (2024-09-06)

### Documentation

 - <csr-id-16e3a0b5a0d8478ade2a3e2de43b9d850d052e51/> include all features on docs generation
 - <csr-id-d5c35c9611482385221bd21f9a5e759b133537ed/> add more lints & fix missing intra links

### New Features

 - <csr-id-e7a5a934be10e0b0be5e8cba767d5845a33bb07d/> Add chapter ordering with function provided by backends
   This enables multiple things:
   - ordering the chapters (obvy);
   - saving the chapters on disk & later merging with online chapters and
     still being able to order them, for instance chronologically;
   
   I think this feature will still require some work, as i'm not sure it's
   convenient enough to use. Moreover, we still use the chapter index in
   the chapter list to deterime its index.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
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

