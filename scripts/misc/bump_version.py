#!/usr/bin/env python3
import sys
import re
import os
import subprocess

def bump_cargo_toml(new_version):
    path = 'Cargo.toml'
    if not os.path.exists(path):
        print(f"Skipping {path} (not found)")
        return

    with open(path, 'r') as f:
        content = f.read()
    
    # Update workspace version
    # Matches [workspace.package] followed by version = "..."
    new_content = re.sub(
        r'(\[workspace\.package\][\s\S]*?version\s*=\s*")([^"]+)(")',
        r'\g<1>' + new_version + r'\g<3>',
        content,
        count=1
    )
    
    with open(path, 'w') as f:
        f.write(new_content)
    print(f"Updated {path}")

def bump_npm_package(path, new_version, update_dep=False):
    if not os.path.exists(path):
        print(f"Skipping {path} (not found)")
        return

    with open(path, 'r') as f:
        content = f.read()
    
    # Update package version (assume it's the first "version": "..." key)
    content = re.sub(
        r'("version":\s*")([^"]+)(")',
        r'\g<1>' + new_version + r'\g<3>',
        content,
        count=1 
    )
    
    if update_dep:
        # Update optionalDependencies ply2splat-native to use caret version
        content = re.sub(
            r'("ply2splat-native":\s*"\^)([^"]+)(")',
            r'\g<1>' + new_version + r'\g<3>',
            content
        )

    with open(path, 'w') as f:
        f.write(content)
    print(f"Updated {path}")

def bump_pyproject(new_version):
    path = 'pyproject.toml'
    if not os.path.exists(path):
        print(f"Skipping {path} (not found)")
        return

    with open(path, 'r') as f:
        content = f.read()
        
    # Update project version
    new_content = re.sub(
        r'(version\s*=\s*")([^"]+)(")',
        r'\g<1>' + new_version + r'\g<3>',
        content,
        count=1
    )
    
    with open(path, 'w') as f:
        f.write(new_content)
    print(f"Updated {path}")

def run_git_commands(version, files):
    try:
        # Stage specific changes
        subprocess.run(["git", "add"] + files, check=True)
        
        # Check for staged changes
        # git diff --cached --quiet returns 1 if there are differences (changes), 0 if clean
        result = subprocess.run(["git", "diff", "--cached", "--quiet"])
        
        if result.returncode == 1: # Changes detected
            commit_msg = f"chore: bump version to {version}"
            subprocess.run(["git", "commit", "-m", commit_msg], check=True)
            print(f"Committed changes: {commit_msg}")
        else:
            print("Nothing to commit (files might remain unchanged).")
        
        # Tag
        tag_name = f"v{version}"
        # Check if tag exists
        tag_check = subprocess.run(["git", "tag", "-l", tag_name], capture_output=True, text=True)
        if tag_name not in tag_check.stdout:
            subprocess.run(["git", "tag", tag_name], check=True)
            print(f"Created tag: {tag_name}")
        else:
            print(f"Tag {tag_name} already exists.")

    except subprocess.CalledProcessError as e:
        print(f"Git command failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <new_version>")
        sys.exit(1)
    
    new_version = sys.argv[1]
    
    # Simple validation for X.Y.Z
    if not re.match(r'^\d+\.\d+\.\d+$', new_version):
        print("Error: Version must be in format X.Y.Z")
        sys.exit(1)

    print(f"Bumping version to {new_version}...")
    
    files_to_update = [
        'Cargo.toml',
        'packages/ply2splat/package.json',
        'bindings/ply2splat-napi/package.json',
        'pyproject.toml'
    ]

    try:
        bump_cargo_toml(new_version)
        bump_npm_package('packages/ply2splat/package.json', new_version, update_dep=True)
        bump_npm_package('bindings/ply2splat-napi/package.json', new_version)
        bump_pyproject(new_version)
        print(f"Successfully updated files to {new_version}")
        
        run_git_commands(new_version, files_to_update)

        # Ask to push
        response = input("Do you want to push changes and tags now? [y/N] ")
        if response.lower() == 'y':
            print("Pushing changes...")
            subprocess.run(["git", "push"], check=True)
            tag_name = f"v{new_version}"
            print(f"Pushing tag {tag_name}...")
            subprocess.run(["git", "push", "origin", tag_name], check=True)
            print("Done.")
        else:
            print(f"Skipping push. Remember to run: git push && git push origin v{new_version}")
        
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)