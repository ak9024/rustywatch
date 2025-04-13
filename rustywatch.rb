class Rustywatch < Formula
  desc "Live reloading for any programming languages with process monitoring"
  homepage "https://rustywatch.vercel.app/"
  url "https://github.com/ak9024/rustywatch/archive/refs/tags/v0.3.0.tar.gz"
  sha256 "1e7cdf26dcaddcb28187a3c96e9f83023173c819be8235430ad90101e2611f5f"
  license "MIT OR Apache-2.0"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--locked", "--root", prefix, "--path", "."
  end

  test do
    assert_match "rustywatch 0.3.0", shell_output("#{bin}/rustywatch --version")
  end
end