#!/usr/bin/env ruby
# Point the WebDriverAgentRunner target at the fastlane match profile using manual
# signing, so Appium can build WebDriverAgent for real iOS devices in CI.
#
# This has to be a per-target patch of the WDA Xcode project: passing the profile
# globally (xcodebuild command line or -xcconfig) also applies it to the
# WebDriverAgentLib framework target, which cannot take provisioning profiles and
# fails the build. Leaving the project on automatic signing does not work either,
# because automatic signing only considers Xcode-managed profiles (which xcodebuild
# cannot generate without an Apple ID session), never the installed match profile.
#
# Appium rewrites this project on driver (re)install, so run this before every test
# job; it is idempotent.
require 'xcodeproj'

project_path = ARGV[0] || File.join(
  Dir.home,
  '.appium/node_modules/appium-xcuitest-driver/node_modules/appium-webdriveragent/WebDriverAgent.xcodeproj'
)
abort "WDA project not found at #{project_path}" unless File.exist?(project_path)

project = Xcodeproj::Project.open(project_path)
target = project.targets.find { |t| t.name == 'WebDriverAgentRunner' } ||
         abort("WebDriverAgentRunner target not found in #{project_path}")

target.build_configurations.each do |config|
  config.build_settings['CODE_SIGN_STYLE'] = 'Manual'
  config.build_settings['DEVELOPMENT_TEAM'] = 'XGL6UKBPLP'
  config.build_settings['CODE_SIGN_IDENTITY'] = 'Apple Development'
  config.build_settings['PROVISIONING_PROFILE_SPECIFIER'] =
    'match Development nl.ictu.edi.wallet.web-driver-agent-runner.xctrunner'
end

project.save
puts "Patched #{project_path}: WebDriverAgentRunner uses the match profile with manual signing"
