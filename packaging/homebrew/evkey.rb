# Cask template. The release workflow (.github/workflows/release.yml, job `macos`) rewrites
# `version` and `sha256` in ngocnc/homebrew-evkey/Casks/evkey.rb on every stable tag.
cask "evkey" do
  version "2.0.0"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"

  url "https://github.com/ngocnc/EVKey/releases/download/v#{version}/EVKey-#{version}-universal.dmg"
  name "EVKey"
  desc "Vietnamese input method (Telex, VNI) for the macOS menu bar"
  homepage "https://github.com/ngocnc/EVKey"

  depends_on macos: ">= :ventura"

  app "EVKey.app"

  # Ad-hoc signed build: strip quarantine so Gatekeeper does not block the first launch.
  postflight do
    system_command "/usr/bin/xattr",
                   args: ["-dr", "com.apple.quarantine", "#{appdir}/EVKey.app"]
  end

  zap trash: [
    "~/Library/Application Support/EVKey",
    "~/Library/LaunchAgents/com.chrisnguyen.evkey.plist",
  ]
end
