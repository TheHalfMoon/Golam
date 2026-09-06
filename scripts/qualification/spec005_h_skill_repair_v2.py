from pathlib import Path

path = Path("crates/golamd/src/skill_packages.rs")
content = path.read_text()
old = '''    descriptor.validate()?;

    Ok(ReviewedInstructionSkill {
'''
new = '''    descriptor.validate()?;

    // Freeze a package identity only when a complete second bounded scan matches the first.
    // This prevents a reviewed digest from representing a mixed state assembled while files or
    // directory entries were changing during discovery. Activation still performs its own live
    // rediscovery immediately before use.
    let stable_files = collect_package_files(&canonical_root)?;
    if stable_files != files {
        return Err(SkillPackageError::PackageChangedDuringDiscovery);
    }

    Ok(ReviewedInstructionSkill {
'''
if old not in content:
    raise SystemExit("expected descriptor validation return point not found")
path.write_text(content.replace(old, new, 1))
