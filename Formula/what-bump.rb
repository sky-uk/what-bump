class WhatBump < Formula
  version "0.1.0"
  desc "Detect required version bump based on conventional commit messages"
  homepage "https://github.com/sky-uk/what-bump"

  if OS.mac?
      url "https://github.com/sky-uk/wht-bump/releases/download/#{version}/ripgrep-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "???"
  elsif OS.linux?
      url "https://github.com/sky-uk/wht-bump/releases/download/#{version}/ripgrep-#{version}-x86_64-unknown-linux-musl.tar.gz"
      sha256 "???"
  end

  conflicts_with "what-bump"

  def install
    bin.install "what-bump"
  end

end
