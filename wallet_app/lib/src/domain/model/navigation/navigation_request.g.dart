// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'navigation_request.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

GenericNavigationRequest _$GenericNavigationRequestFromJson(
  Map<String, dynamic> json,
) => GenericNavigationRequest(
  json['destination'] as String,
  removeUntil: json['removeUntil'] as String?,
  argument: json['argument'],
  navigatePrerequisites:
      (json['navigatePrerequisites'] as List<dynamic>?)
          ?.map((e) => $enumDecode(_$NavigationPrerequisiteEnumMap, e))
          .toList() ??
      const [],
  preNavigationActions:
      (json['preNavigationActions'] as List<dynamic>?)
          ?.map((e) => $enumDecode(_$PreNavigationActionEnumMap, e))
          .toList() ??
      const [],
);

Map<String, dynamic> _$GenericNavigationRequestToJson(
  GenericNavigationRequest instance,
) => <String, dynamic>{
  'destination': instance.destination,
  'removeUntil': instance.removeUntil,
  'argument': instance.argument,
  'navigatePrerequisites': instance.navigatePrerequisites.map((e) => _$NavigationPrerequisiteEnumMap[e]!).toList(),
  'preNavigationActions': instance.preNavigationActions.map((e) => _$PreNavigationActionEnumMap[e]!).toList(),
};

const _$NavigationPrerequisiteEnumMap = {
  NavigationPrerequisite.walletUnlocked: 'walletUnlocked',
  NavigationPrerequisite.walletInitialized: 'walletInitialized',
  NavigationPrerequisite.walletInReadyState: 'walletInReadyState',
  NavigationPrerequisite.pidInitialized: 'pidInitialized',
};

const _$PreNavigationActionEnumMap = {
  PreNavigationAction.setupMockedWallet: 'setupMockedWallet',
  PreNavigationAction.disableUpcomingPageTransition: 'disableUpcomingPageTransition',
};
