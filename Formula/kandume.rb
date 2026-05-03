class Kandume < Formula
  desc "Terminal multiplexer TUI (Ratatui, tmux-style Ctrl-b)"
  homepage "https://github.com/tarabakz25/kandume"
  url "https://github.com/tarabakz25/kandume/archive/refs/tags/v0.1.1.tar.gz"
  sha256 "0722d16261ca4306cd72312753faa6184d65fc70c3ff43df306aee58826687dd"
  head "https://github.com/tarabakz25/kandume.git", branch: "develop"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/kandume --version")
  end
end
