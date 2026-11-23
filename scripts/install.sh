#!/usr/bin/env bash
# ==============================================================================
# GitVerse Cross-Platform One-Line Installer (Linux & macOS)
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/farshidrezaei/gitverse/main/scripts/install.sh | bash
#
# Options:
#   --version <tag>     Install specific release tag (e.g. v1.0.0)
#   --force             Bypass confirmation prompts
#   --reinstall         Force re-installation even if current version is up to date
#   --uninstall         Uninstall existing GitVerse desktop installation
#   --help              Display help information
# ==============================================================================

set -euo pipefail

# Visual styling
COLOR_RESET="\033[0m"
COLOR_BOLD="\033[1m"
COLOR_GREEN="\033[32m"
COLOR_BLUE="\033[34m"
COLOR_CYAN="\033[36m"
COLOR_YELLOW="\033[33m"
COLOR_RED="\033[31m"

REPOSITORY_OWNER="farshidrezaei"
REPOSITORY_NAME="gitverse"
GITHUB_REPOSITORY="${REPOSITORY_OWNER}/${REPOSITORY_NAME}"

# State variables
SPECIFIED_VERSION=""
IS_FORCE_MODE=false
IS_REINSTALL_MODE=false
IS_UNINSTALL_MODE=false

print_banner() {
    printf "${COLOR_CYAN}${COLOR_BOLD}"
    cat << "EOF"
   ____ _ _   _    __
  / ___(_) |_| |  / /__ _ __ ___  ___
 | |  _| | __| | / / _ \ '__/ __|/ _ \
 | |_| | | |_| |/ /  __/ |  \__ \  __/
  \____|_|\__|_/_/ \___|_|  |___/\___|

EOF
    printf "${COLOR_RESET}"
    printf "${COLOR_BOLD}GitVerse Desktop Client Installer${COLOR_RESET}\n"
    printf "Fast, visual, local-first Git client for Linux and macOS\n\n"
}

log_info() {
    local message="$1"
    printf "${COLOR_BLUE}${COLOR_BOLD}==>${COLOR_RESET} %s\n" "${message}"
}

log_success() {
    local message="$1"
    printf "${COLOR_GREEN}${COLOR_BOLD}==>${COLOR_RESET} %s\n" "${message}"
}

log_warning() {
    local message="$1"
    printf "${COLOR_YELLOW}${COLOR_BOLD}Warning:${COLOR_RESET} %s\n" "${message}"
}

log_error() {
    local message="$1"
    printf "${COLOR_RED}${COLOR_BOLD}Error:${COLOR_RESET} %s\n" "${message}" >&2
}

display_help() {
    cat << EOF
GitVerse Installer (Linux & macOS)

Usage:
  install.sh [options]

Options:
  -v, --version <tag>   Install a specific version (e.g., v1.0.0)
  -f, --force           Bypass interactive confirmation prompts
      --reinstall       Reinstall GitVerse even if latest version is already installed
      --uninstall       Remove GitVerse desktop application and symlinks
  -h, --help            Show this help message and exit

Examples:
  curl -fsSL https://gitverse.dev/install.sh | bash
  curl -fsSL https://gitverse.dev/install.sh | bash -s -- --force
  curl -fsSL https://gitverse.dev/install.sh | bash -s -- --version v1.0.0
  curl -fsSL https://gitverse.dev/install.sh | bash -s -- --uninstall
EOF
}

parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -v|--version)
                if [[ -z "${2:-}" ]]; then
                    log_error "Option --version requires a version argument"
                    exit 1
                fi
                SPECIFIED_VERSION="$2"
                shift 2
                ;;
            -f|--force)
                IS_FORCE_MODE=true
                shift
                ;;
            --reinstall)
                IS_REINSTALL_MODE=true
                shift
                ;;
            --uninstall)
                IS_UNINSTALL_MODE=true
                shift
                ;;
            -h|--help)
                display_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                display_help
                exit 1
                ;;
        esac
    done
}

detect_operating_system() {
    local operating_system_raw
    operating_system_raw="$(uname -s)"
    case "${operating_system_raw}" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
        *)
            log_error "Unsupported operating system: ${operating_system_raw}. GitVerse installer supports Linux and macOS."
            exit 1
            ;;
    esac
}

detect_architecture() {
    local architecture_raw
    architecture_raw="$(uname -m)"
    case "${architecture_raw}" in
        x86_64|amd64)   echo "x86_64" ;;
        arm64|aarch64)  echo "aarch64" ;;
        *)
            log_error "Unsupported architecture: ${architecture_raw}. GitVerse supports x86_64 and aarch64."
            exit 1
            ;;
    esac
}

prompt_user_confirmation() {
    local prompt_message="$1"
    local default_choice="${2:-N}"

    if [[ "${IS_FORCE_MODE}" == true ]]; then
        return 0
    fi

    # Read from /dev/tty if stdin is piped (e.g. curl | bash)
    local user_response=""
    if [[ -t 0 ]]; then
        read -r -p "${prompt_message} " user_response || true
    elif [[ -c /dev/tty ]]; then
        read -r -p "${prompt_message} " user_response < /dev/tty || true
    else
        # Non-interactive shell without tty
        if [[ "${default_choice}" == "Y" || "${default_choice}" == "y" ]]; then
            return 0
        else
            return 1
        fi
    fi

    if [[ -z "${user_response}" ]]; then
        user_response="${default_choice}"
    fi

    case "${user_response}" in
        [Yy]*) return 0 ;;
        *) return 1 ;;
    esac
}

get_installed_version() {
    local operating_system="$1"

    # 1. Check if gitverse command is in PATH
    if command -v gitverse >/dev/null 2>&1; then
        local version_output
        version_output="$(gitverse --version 2>/dev/null || true)"
        if [[ -n "${version_output}" ]]; then
            # Extract semver string like 1.0.0
            echo "${version_output}" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -n 1
            return 0
        fi
    fi

    # 2. Check local directories
    if [[ "${operating_system}" == "macos" ]]; then
        if [[ -d "/Applications/GitVerse.app" ]]; then
            local plist_path="/Applications/GitVerse.app/Contents/Info.plist"
            if [[ -f "${plist_path}" ]] && command -v defaults >/dev/null 2>&1; then
                defaults read "/Applications/GitVerse.app/Contents/Info.plist" CFBundleShortVersionString 2>/dev/null || true
                return 0
            fi
            echo "installed"
            return 0
        fi
    else
        local local_binary="${HOME}/.local/bin/gitverse"
        if [[ -x "${local_binary}" ]]; then
            local version_output
            version_output="$("${local_binary}" --version 2>/dev/null || true)"
            if [[ -n "${version_output}" ]]; then
                echo "${version_output}" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -n 1
                return 0
            fi
            echo "installed"
            return 0
        fi
    fi

    echo ""
}

clean_version_tag() {
    local tag="$1"
    echo "${tag#v}"
}

compare_versions() {
    local installed_version="$1"
    local target_version="$2"

    installed_version="$(clean_version_tag "${installed_version}")"
    target_version="$(clean_version_tag "${target_version}")"

    if [[ "${installed_version}" == "${target_version}" ]]; then
        echo "equal"
        return 0
    fi

    # Compare numeric components
    local installed_major installed_minor installed_patch
    local target_major target_minor target_patch

    IFS='.' read -r installed_major installed_minor installed_patch <<< "${installed_version}"
    IFS='.' read -r target_major target_minor target_patch <<< "${target_version}"

    installed_major="${installed_major:-0}"
    installed_minor="${installed_minor:-0}"
    installed_patch="${installed_patch:-0}"
    target_major="${target_major:-0}"
    target_minor="${target_minor:-0}"
    target_patch="${target_patch:-0}"

    if (( 10#${installed_major} < 10#${target_major} )); then
        echo "older"
    elif (( 10#${installed_major} > 10#${target_major} )); then
        echo "newer"
    elif (( 10#${installed_minor} < 10#${target_minor} )); then
        echo "older"
    elif (( 10#${installed_minor} > 10#${target_minor} )); then
        echo "newer"
    elif (( 10#${installed_patch} < 10#${target_patch} )); then
        echo "older"
    elif (( 10#${installed_patch} > 10#${target_patch} )); then
        echo "newer"
    else
        echo "equal"
    fi
}

fetch_release_information() {
    local requested_tag="$1"
    local api_endpoint

    if [[ -n "${requested_tag}" ]]; then
        # Ensure tag starts with v
        if [[ "${requested_tag}" != v* ]]; then
            requested_tag="v${requested_tag}"
        fi
        api_endpoint="https://api.github.com/repos/${GITHUB_REPOSITORY}/releases/tags/${requested_tag}"
    else
        api_endpoint="https://api.github.com/repos/${GITHUB_REPOSITORY}/releases/latest"
    fi

    local curl_command="curl -fsSL"
    if [[ -n "${GITHUB_TOKEN:-}" ]]; then
        curl_command="curl -fsSL -H \"Authorization: Bearer ${GITHUB_TOKEN}\""
    fi

    local release_json
    release_json="$(eval "${curl_command}" "${api_endpoint}" 2>/dev/null || true)"

    if [[ -z "${release_json}" ]] || [[ "${release_json}" == *"Not Found"* ]]; then
        log_error "Could not fetch release information from GitHub (${api_endpoint})"
        log_error "Please verify your internet connection or repository release status."
        exit 1
    fi

    echo "${release_json}"
}

extract_json_field() {
    local json_content="$1"
    local field_name="$2"

    if command -v jq >/dev/null 2>&1; then
        echo "${json_content}" | jq -r --arg field "${field_name}" '.[$field] // empty'
    else
        # Fallback regex extraction
        echo "${json_content}" | grep -oE "\"${field_name}\":\\s*\"[^\"]+\"" | head -n 1 | cut -d'"' -f4
    fi
}

extract_download_url() {
    local json_content="$1"
    local filename_pattern="$2"

    if command -v jq >/dev/null 2>&1; then
        echo "${json_content}" | jq -r --arg pattern "${filename_pattern}" '.assets[] | select((.name | test($pattern; "i")) and (.name | test("\\.sig$") | not)) | .browser_download_url' | head -n 1
    else
        echo "${json_content}" | grep -oE '"browser_download_url":\s*"[^"]+' | cut -d'"' -f4 | grep -iE "${filename_pattern}" | grep -v '\.sig$' | head -n 1
    fi
}

uninstall_gitverse() {
    local operating_system="$1"
    log_info "Uninstalling GitVerse..."

    if [[ "${operating_system}" == "macos" ]]; then
        if [[ -d "/Applications/GitVerse.app" ]]; then
            rm -rf "/Applications/GitVerse.app"
            log_success "Removed /Applications/GitVerse.app"
        fi
        if [[ -L "/usr/local/bin/gitverse" ]]; then
            rm -f "/usr/local/bin/gitverse"
        fi
        if [[ -L "${HOME}/.local/bin/gitverse" ]]; then
            rm -f "${HOME}/.local/bin/gitverse"
        fi
    else
        # Linux cleanup
        rm -f "${HOME}/.local/bin/gitverse"
        rm -rf "${HOME}/.local/share/gitverse"
        rm -f "${HOME}/.local/share/applications/gitverse.desktop"
        rm -f "${HOME}/.local/share/icons/hicolor/512x512/apps/gitverse.png"
        log_success "Removed GitVerse executable, desktop entry, and icon"

        if command -v update-desktop-database >/dev/null 2>&1; then
            update-desktop-database "${HOME}/.local/share/applications" 2>/dev/null || true
        fi
    fi

    log_success "GitVerse uninstalled successfully."
    exit 0
}

install_linux() {
    local target_version="$1"
    local release_json="$2"
    local architecture="$3"

    log_info "Preparing Linux installation for architecture: ${architecture}..."

    # Look for AppImage asset
    local download_url
    download_url="$(extract_download_url "${release_json}" "\\.AppImage$")"

    if [[ -z "${download_url}" ]]; then
        # Fallback to deb if AppImage not found
        download_url="$(extract_download_url "${release_json}" "amd64\\.deb$")"
    fi

    if [[ -z "${download_url}" ]]; then
        log_error "Could not find a compatible Linux package (AppImage/deb) in release ${target_version}"
        exit 1
    fi

    local temporary_directory
    temporary_directory="$(mktemp -d)"
    local target_appimage_path="${HOME}/.local/share/gitverse/GitVerse-${target_version}.AppImage"
    local bin_symlink="${HOME}/.local/bin/gitverse"

    mkdir -p "${HOME}/.local/bin"
    mkdir -p "${HOME}/.local/share/gitverse"
    mkdir -p "${HOME}/.local/share/applications"
    mkdir -p "${HOME}/.local/share/icons/hicolor/512x512/apps"

    log_info "Downloading GitVerse ${target_version}..."
    printf "URL: %s\n" "${download_url}"

    local download_file="${temporary_directory}/gitverse.AppImage"
    curl -fSL --progress-bar "${download_url}" -o "${download_file}"

    # Move to destination
    mv "${download_file}" "${target_appimage_path}"
    chmod +x "${target_appimage_path}"

    # Create symlink
    ln -sf "${target_appimage_path}" "${bin_symlink}"

    # Download or place official application icon
    local icon_destination="${HOME}/.local/share/icons/hicolor/512x512/apps/gitverse.png"
    local icon_url="https://raw.githubusercontent.com/${GITHUB_REPOSITORY}/main/docs/assets/gitverse-logo.png"
    curl -fsSL "${icon_url}" -o "${icon_destination}" 2>/dev/null || true

    # Create desktop launcher
    local desktop_entry_path="${HOME}/.local/share/applications/gitverse.desktop"
    cat > "${desktop_entry_path}" << EOF
[Desktop Entry]
Name=GitVerse
GenericName=Git Desktop Client
Comment=Fast, Visual, and Local-First Git Desktop Client
Exec=${bin_symlink} %U
Icon=gitverse
Terminal=false
Type=Application
Categories=Development;RevisionControl;
StartupWMClass=GitVerse
MimeType=x-scheme-handler/gitverse;
EOF
    chmod +x "${desktop_entry_path}"

    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "${HOME}/.local/share/applications" 2>/dev/null || true
    fi

    # Clean up temporary files
    rm -rf "${temporary_directory}"

    # Verify PATH
    local is_bin_in_path=false
    if [[ ":${PATH}:" == *":${HOME}/.local/bin:"* ]]; then
        is_bin_in_path=true
    fi

    printf "\n"
    log_success "GitVerse ${target_version} installed successfully!"
    printf "  • Binary:  %s\n" "${bin_symlink}"
    printf "  • Desktop: %s\n" "${desktop_entry_path}"

    if [[ "${is_bin_in_path}" == false ]]; then
        printf "\n"
        log_warning "${HOME}/.local/bin is not in your current PATH environment."
        printf "To launch GitVerse from terminal, add this to your ~/.bashrc or ~/.zshrc:\n"
        printf "  export PATH=\"\$HOME/.local/bin:\$PATH\"\n"
    fi
}

install_macos() {
    local target_version="$1"
    local release_json="$2"
    local architecture="$3"

    log_info "Preparing macOS installation for architecture: ${architecture}..."

    # Look for DMG asset
    local download_url
    download_url="$(extract_download_url "${release_json}" "\\.dmg$")"

    if [[ -z "${download_url}" ]]; then
        log_error "Could not find macOS DMG installer in release ${target_version}"
        exit 1
    fi

    local temporary_directory
    temporary_directory="$(mktemp -d)"
    local downloaded_dmg="${temporary_directory}/GitVerse-${target_version}.dmg"
    local mount_point="${temporary_directory}/mount"

    log_info "Downloading GitVerse ${target_version} DMG..."
    printf "URL: %s\n" "${download_url}"
    curl -fSL --progress-bar "${download_url}" -o "${downloaded_dmg}"

    log_info "Mounting disk image..."
    mkdir -p "${mount_point}"
    hdiutil attach -nobrowse -readonly "${downloaded_dmg}" -mountpoint "${mount_point}" >/dev/null

    local app_source_path="${mount_point}/GitVerse.app"
    if [[ ! -d "${app_source_path}" ]]; then
        # Search for any .app in mounted volume
        app_source_path="$(find "${mount_point}" -maxdepth 2 -name "*.app" | head -n 1)"
    fi

    if [[ -z "${app_source_path}" || ! -d "${app_source_path}" ]]; then
        hdiutil detach "${mount_point}" -force >/dev/null 2>&1 || true
        rm -rf "${temporary_directory}"
        log_error "Could not locate GitVerse.app inside DMG"
        exit 1
    fi

    log_info "Installing to /Applications/GitVerse.app..."
    local destination_directory="/Applications"
    if [[ ! -w "${destination_directory}" ]]; then
        destination_directory="${HOME}/Applications"
        mkdir -p "${destination_directory}"
    fi

    rm -rf "${destination_directory}/GitVerse.app"
    cp -R "${app_source_path}" "${destination_directory}/GitVerse.app"

    # Unmount DMG
    hdiutil detach "${mount_point}" -force >/dev/null 2>&1 || true
    rm -rf "${temporary_directory}"

    # Setup CLI launcher symlink
    mkdir -p "${HOME}/.local/bin"
    local launcher_binary="${destination_directory}/GitVerse.app/Contents/MacOS/GitVerse"
    if [[ -f "${launcher_binary}" ]]; then
        ln -sf "${launcher_binary}" "${HOME}/.local/bin/gitverse"
    fi

    printf "\n"
    log_success "GitVerse ${target_version} installed successfully!"
    printf "  • Application: %s/GitVerse.app\n" "${destination_directory}"
    printf "  • CLI:         %s/.local/bin/gitverse\n" "${HOME}"
}

main() {
    parse_arguments "$@"
    print_banner

    local operating_system
    operating_system="$(detect_operating_system)"

    if [[ "${IS_UNINSTALL_MODE}" == true ]]; then
        uninstall_gitverse "${operating_system}"
        return 0
    fi

    local architecture
    architecture="$(detect_architecture)"

    log_info "Detected OS: ${operating_system} (${architecture})"

    # Fetch release metadata from GitHub
    log_info "Checking available releases on GitHub..."
    local release_json
    release_json="$(fetch_release_information "${SPECIFIED_VERSION}")"

    local target_tag
    target_tag="$(extract_json_field "${release_json}" "tag_name")"
    if [[ -z "${target_tag}" ]]; then
        log_error "Unable to parse release tag from GitHub"
        exit 1
    fi

    local target_version
    target_version="$(clean_version_tag "${target_tag}")"

    # Detect current local installation
    local current_installed_version
    current_installed_version="$(get_installed_version "${operating_system}")"

    if [[ -n "${current_installed_version}" ]]; then
        local version_status
        version_status="$(compare_versions "${current_installed_version}" "${target_version}")"

        if [[ "${version_status}" == "equal" ]]; then
            printf "\n"
            printf "${COLOR_GREEN}${COLOR_BOLD}GitVerse is already installed and up to date (v%s).${COLOR_RESET}\n" "${current_installed_version}"

            if [[ "${IS_REINSTALL_MODE}" == true ]]; then
                log_info "Reinstalling GitVerse v${target_version} as requested..."
            else
                if prompt_user_confirmation "Would you like to reinstall GitVerse v${target_version}? [y/N]:" "N"; then
                    log_info "Reinstalling GitVerse v${target_version}..."
                else
                    log_info "Installation aborted by user. Exiting."
                    exit 0
                fi
            fi
        elif [[ "${version_status}" == "older" ]]; then
            printf "\n"
            printf "${COLOR_YELLOW}${COLOR_BOLD}Update Available!${COLOR_RESET}\n"
            printf "Installed version: v%s\n" "${current_installed_version}"
            printf "Latest version:    v%s\n\n" "${target_version}"

            if prompt_user_confirmation "Would you like to update GitVerse to v${target_version} now? [Y/n]:" "Y"; then
                log_info "Updating GitVerse from v${current_installed_version} to v${target_version}..."
            else
                log_info "Update skipped by user. Exiting."
                exit 0
            fi
        else
            printf "\n"
            log_warning "Installed version (v${current_installed_version}) is newer than target release (v${target_version})."
            if prompt_user_confirmation "Do you want to downgrade to v${target_version}? [y/N]:" "N"; then
                log_info "Downgrading to GitVerse v${target_version}..."
            else
                log_info "Operation cancelled by user. Exiting."
                exit 0
            fi
        fi
    else
        log_info "No existing GitVerse installation detected."
        log_info "Installing GitVerse v${target_version}..."
    fi

    # Dispatch to platform-specific installer
    if [[ "${operating_system}" == "macos" ]]; then
        install_macos "${target_version}" "${release_json}" "${architecture}"
    else
        install_linux "${target_version}" "${release_json}" "${architecture}"
    fi

    printf "\n"
    printf "${COLOR_CYAN}Enjoying GitVerse? Star the repository: https://github.com/${GITHUB_REPOSITORY}${COLOR_RESET}\n"
}

main "$@"
