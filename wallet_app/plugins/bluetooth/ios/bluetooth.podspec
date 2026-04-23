#
# To learn more about a Podspec see http://guides.cocoapods.org/syntax/podspec.html.
# Run `pod lib lint bluetooth.podspec` to validate before publishing.
#
Pod::Spec.new do |s|
  s.name             = 'bluetooth'
  s.version          = '0.1.0'
  s.summary          = 'A Flutter bluetooth plugin.'
  s.description      = <<-DESC
A Flutter bluetooth plugin.
                       DESC
  s.homepage         = 'https://github.com/MinBZK/nl-wallet'
  s.license          = { :file => '../LICENSE' }
  s.author           = { 'NL Wallet' => 'email@example.org' }
  s.source           = { :path => '.' }
  s.source_files = 'Classes/**/*'
  s.dependency 'Flutter'
  s.platform = :ios, '13.0'

  # Flutter.framework does not contain a i386 slice.
  s.pod_target_xcconfig = { 'DEFINES_MODULE' => 'YES', 'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'i386' }
  s.swift_version = '5.0'

  # If your plugin requires a privacy manifest, for example if it uses any
  # required reason APIs, update the PrivacyInfo.xcprivacy file to describe your
  # plugin's privacy impact, and then uncomment this line. For more information,
  # see https://developer.apple.com/documentation/bundleresources/privacy_manifest_files
  # s.resource_bundles = {'bluetooth_privacy' => ['Resources/PrivacyInfo.xcprivacy']}
end
