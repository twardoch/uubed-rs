#!/bin/bash
# this_file: version-manager.sh
# Version management script for git-tag-based semversioning

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_usage() {
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  current         Show current version"
    echo "  next            Show next version for each increment type"
    echo "  bump TYPE       Bump version (major|minor|patch|prerelease)"
    echo "  tag             Create and push git tag for current version"
    echo "  release TYPE    Bump version and create release tag"
    echo "  validate        Validate current version setup"
    echo ""
    echo "Options:"
    echo "  -n, --dry-run   Show what would be done without making changes"
    echo "  -f, --force     Force operation even if checks fail"
    echo "  -h, --help      Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 current                    # Show current version"
    echo "  $0 next                       # Show next versions"
    echo "  $0 bump patch                 # Increment patch version"
    echo "  $0 release minor              # Bump minor version and create release"
    echo "  $0 tag                        # Create git tag for current version"
}

# Get current version from git tags
get_current_version() {
    local version
    
    # Try to get version from git tags
    if git describe --tags --exact-match HEAD 2>/dev/null; then
        version=$(git describe --tags --exact-match HEAD 2>/dev/null | sed 's/^v//')
    else
        # Get latest tag
        local latest_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
        version=$(echo "$latest_tag" | sed 's/^v//')
        log_warning "Not on a tagged commit. Latest tag: $latest_tag" >&2
    fi
    
    echo "$version"
}

# Get version from Cargo.toml
get_cargo_version() {
    cd rust && cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'
}

# Parse semantic version
parse_version() {
    local version="$1"
    local -A parts
    
    # Split version by dots
    IFS='.' read -ra VERSION_PARTS <<< "$version"
    
    if [ ${#VERSION_PARTS[@]} -lt 3 ]; then
        log_error "Invalid version format: $version"
        return 1
    fi
    
    parts[major]=${VERSION_PARTS[0]}
    parts[minor]=${VERSION_PARTS[1]}
    
    # Handle patch version with prerelease
    if [[ ${VERSION_PARTS[2]} == *"-"* ]]; then
        IFS='-' read -ra PATCH_PARTS <<< "${VERSION_PARTS[2]}"
        parts[patch]=${PATCH_PARTS[0]}
        parts[prerelease]=${PATCH_PARTS[1]}
    else
        parts[patch]=${VERSION_PARTS[2]}
        parts[prerelease]=""
    fi
    
    # Return as associative array (bash limitation workaround)
    echo "${parts[major]} ${parts[minor]} ${parts[patch]} ${parts[prerelease]}"
}

# Increment version
increment_version() {
    local version="$1"
    local increment_type="$2"
    
    # Parse current version
    local parsed
    parsed=$(parse_version "$version")
    read -r major minor patch prerelease <<< "$parsed"
    
    case "$increment_type" in
        major)
            major=$((major + 1))
            minor=0
            patch=0
            prerelease=""
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            prerelease=""
            ;;
        patch)
            patch=$((patch + 1))
            prerelease=""
            ;;
        prerelease)
            if [ -n "$prerelease" ]; then
                # Increment existing prerelease
                if [[ "$prerelease" == *"."* ]]; then
                    IFS='.' read -ra PRE_PARTS <<< "$prerelease"
                    prerelease="${PRE_PARTS[0]}.$((${PRE_PARTS[1]} + 1))"
                else
                    prerelease="$prerelease.1"
                fi
            else
                # Create new prerelease
                patch=$((patch + 1))
                prerelease="pre.0"
            fi
            ;;
        *)
            log_error "Invalid increment type: $increment_type"
            return 1
            ;;
    esac
    
    # Construct new version
    local new_version="$major.$minor.$patch"
    if [ -n "$prerelease" ]; then
        new_version="$new_version-$prerelease"
    fi
    
    echo "$new_version"
}

# Update version in files
update_version_in_files() {
    local version="$1"
    local dry_run="$2"
    
    log_info "Updating version to $version in project files"
    
    if [ "$dry_run" = false ]; then
        # Update workspace version
        sed -i.bak "s/^version = \".*\"/version = \"$version\"/" Cargo.toml
        
        # Update rust package version if needed
        if [ -f "rust/Cargo.toml" ]; then
            if grep -q "version.workspace = true" rust/Cargo.toml; then
                log_info "Rust package uses workspace version"
            else
                sed -i.bak "s/^version = \".*\"/version = \"$version\"/" rust/Cargo.toml
            fi
        fi
        
        # Clean up backup files
        find . -name "*.bak" -delete
        
        log_success "Version updated in files"
    else
        log_info "[DRY RUN] Would update version in Cargo.toml files"
    fi
}

# Create and push git tag
create_git_tag() {
    local version="$1"
    local dry_run="$2"
    
    local tag_name="v$version"
    
    log_info "Creating git tag: $tag_name"
    
    if [ "$dry_run" = false ]; then
        # Check if tag already exists
        if git tag -l | grep -q "^$tag_name$"; then
            log_error "Tag $tag_name already exists"
            return 1
        fi
        
        # Create annotated tag
        git tag -a "$tag_name" -m "Release version $version"
        
        # Push tag to remote
        if git remote get-url origin >/dev/null 2>&1; then
            git push origin "$tag_name"
            log_success "Tag $tag_name created and pushed"
        else
            log_warning "No remote origin found, tag created locally only"
        fi
    else
        log_info "[DRY RUN] Would create and push tag: $tag_name"
    fi
}

# Validate version setup
validate_version_setup() {
    log_info "Validating version setup..."
    
    local issues=0
    
    # Check if git is available
    if ! command -v git >/dev/null 2>&1; then
        log_error "Git is not installed"
        issues=$((issues + 1))
    fi
    
    # Check if we're in a git repository
    if ! git rev-parse --git-dir >/dev/null 2>&1; then
        log_error "Not in a git repository"
        issues=$((issues + 1))
    fi
    
    # Check if Cargo.toml exists
    if [ ! -f "Cargo.toml" ]; then
        log_error "Cargo.toml not found"
        issues=$((issues + 1))
    fi
    
    # Check if jq is available
    if ! command -v jq >/dev/null 2>&1; then
        log_error "jq is not installed (required for parsing Cargo metadata)"
        issues=$((issues + 1))
    fi
    
    # Check current version consistency
    local git_version
    local cargo_version
    
    git_version=$(get_current_version)
    cargo_version=$(get_cargo_version)
    
    if [ "$git_version" != "$cargo_version" ]; then
        log_warning "Version mismatch: Git=$git_version, Cargo=$cargo_version"
        issues=$((issues + 1))
    fi
    
    # Check if working directory is clean
    if ! git diff-index --quiet HEAD --; then
        log_warning "Working directory is not clean"
        issues=$((issues + 1))
    fi
    
    if [ $issues -eq 0 ]; then
        log_success "Version setup is valid"
        return 0
    else
        log_error "Found $issues issue(s) in version setup"
        return 1
    fi
}

# Show current version
show_current_version() {
    local git_version
    local cargo_version
    
    git_version=$(get_current_version)
    cargo_version=$(get_cargo_version)
    
    echo "Current Versions:"
    echo "  Git tag:   $git_version"
    echo "  Cargo.toml: $cargo_version"
    
    if [ "$git_version" = "$cargo_version" ]; then
        echo -e "  Status:    ${GREEN}Consistent${NC}"
    else
        echo -e "  Status:    ${YELLOW}Inconsistent${NC}"
    fi
}

# Show next version options
show_next_versions() {
    local current_version
    current_version=$(get_current_version)
    
    echo "Current version: $current_version"
    echo ""
    echo "Next version options:"
    echo "  Major:      $(increment_version "$current_version" major)"
    echo "  Minor:      $(increment_version "$current_version" minor)"
    echo "  Patch:      $(increment_version "$current_version" patch)"
    echo "  Prerelease: $(increment_version "$current_version" prerelease)"
}

# Main command handling
main() {
    local command=""
    local dry_run=false
    local force=false
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            current|next|bump|tag|release|validate)
                command="$1"
                shift
                ;;
            -n|--dry-run)
                dry_run=true
                shift
                ;;
            -f|--force)
                force=true
                shift
                ;;
            -h|--help)
                print_usage
                exit 0
                ;;
            *)
                if [ -z "$command" ]; then
                    log_error "Unknown command: $1"
                    print_usage
                    exit 1
                else
                    break
                fi
                ;;
        esac
    done
    
    # Handle commands
    case "$command" in
        current)
            show_current_version
            ;;
        next)
            show_next_versions
            ;;
        bump)
            if [ $# -lt 1 ]; then
                log_error "Bump command requires increment type (major|minor|patch|prerelease)"
                exit 1
            fi
            
            local increment_type="$1"
            local current_version
            local new_version
            
            current_version=$(get_current_version)
            new_version=$(increment_version "$current_version" "$increment_type")
            
            if [ "$force" = false ]; then
                validate_version_setup || exit 1
            fi
            
            update_version_in_files "$new_version" "$dry_run"
            
            if [ "$dry_run" = false ]; then
                log_success "Version bumped from $current_version to $new_version"
            else
                log_info "[DRY RUN] Would bump version from $current_version to $new_version"
            fi
            ;;
        tag)
            local current_version
            current_version=$(get_current_version)
            
            if [ "$force" = false ]; then
                validate_version_setup || exit 1
            fi
            
            create_git_tag "$current_version" "$dry_run"
            ;;
        release)
            if [ $# -lt 1 ]; then
                log_error "Release command requires increment type (major|minor|patch|prerelease)"
                exit 1
            fi
            
            local increment_type="$1"
            local current_version
            local new_version
            
            current_version=$(get_current_version)
            new_version=$(increment_version "$current_version" "$increment_type")
            
            if [ "$force" = false ]; then
                validate_version_setup || exit 1
            fi
            
            update_version_in_files "$new_version" "$dry_run"
            
            if [ "$dry_run" = false ]; then
                # Commit version changes
                git add Cargo.toml rust/Cargo.toml 2>/dev/null || true
                git commit -m "Bump version to $new_version"
            fi
            
            create_git_tag "$new_version" "$dry_run"
            
            if [ "$dry_run" = false ]; then
                log_success "Released version $new_version"
            else
                log_info "[DRY RUN] Would release version $new_version"
            fi
            ;;
        validate)
            validate_version_setup
            ;;
        *)
            log_error "No command specified"
            print_usage
            exit 1
            ;;
    esac
}

# Run main function
main "$@"