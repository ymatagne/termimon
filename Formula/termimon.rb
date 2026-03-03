class Termimon < Formula
  desc "Your AI agents, alive in the terminal. Pixel creature companions for tmux."
  homepage "https://github.com/ymatagne/termimon"
  license "MIT"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/ymatagne/termimon/releases/latest/download/termimon-macos-arm64"
      sha256 "PLACEHOLDER"
    else
      url "https://github.com/ymatagne/termimon/releases/latest/download/termimon-macos-x86_64"
      sha256 "PLACEHOLDER"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/ymatagne/termimon/releases/latest/download/termimon-linux-arm64"
      sha256 "PLACEHOLDER"
    else
      url "https://github.com/ymatagne/termimon/releases/latest/download/termimon-linux-x86_64"
      sha256 "PLACEHOLDER"
    end
  end

  def install
    bin.install stable.url.split("/").last => "termimon"
  end

  test do
    assert_match "termimon", shell_output("#{bin}/termimon --version")
  end
end
