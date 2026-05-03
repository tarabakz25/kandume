class Kandume < Formula
  desc "Terminal multiplexer TUI (Ratatui, tmux-style Ctrl-b)"
  homepage "https://github.com/tarabakz25/kandume"
  url "https://github.com/tarabakz25/kandume/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "4442118dc32589dc31c0069b8db988a57127e3c2f45c0856369bda0e0ea1b9cb"
  head "https://github.com/tarabakz25/kandume.git", branch: "develop"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_predicate bin/"kandume", :executable?
  end
end
