// Animation
import 'package:flutter/painting.dart';

const kDefaultAnimationDuration = Duration(milliseconds: 300);

// UI
const double kCardBreakPointWidth = 300; // Used to calculate columns for MasonryGrid
const EdgeInsets kDefaultTitlePadding = EdgeInsets.only(left: 16, right: 16, top: 12, bottom: 8);

// Security
const kPinDigits = 6;
const kMockPin = '123456';

// DigiD
const kDigidWebsiteUrl = 'https://www.digid.nl/inlogmethodes/identiteitsbewijs';

// Mocking
const kDefaultMockDelay = Duration(milliseconds: 1000);
const kDefaultDigidMockDelay = Duration(seconds: 2);
const kDefaultAnnouncementDelay = Duration(milliseconds: 500);

// Deeplink / dive related
const kDeeplinkScheme = 'walletdebuginteraction';
const kDeeplinkHost = 'deeplink'; //Used in our custom deeplinks
const kDeeplinkPath = '/deeplink'; //Used to support deeplinks with dedicated host
const kDeepDiveHost = 'deepdive'; //Used in our custom deepdives
const kDeepDivePath = '/deepdive'; //Used to support deepdives with dedicated host
