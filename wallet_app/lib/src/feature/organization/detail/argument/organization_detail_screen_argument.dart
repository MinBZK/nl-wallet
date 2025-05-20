import 'package:equatable/equatable.dart';
import 'package:flutter/foundation.dart';

import '../../../../domain/model/organization.dart';

@immutable
class OrganizationDetailScreenArgument extends Equatable {
  static const _kOrganizationKey = 'organization';
  static const _kSharedDataBeforeKey = 'shared_data_before';

  final Organization organization;
  final bool sharedDataWithOrganizationBefore;

  const OrganizationDetailScreenArgument({
    required this.organization,
    required this.sharedDataWithOrganizationBefore,
  });

  Map<String, dynamic> toMap() {
    return {
      _kOrganizationKey: organization,
      _kSharedDataBeforeKey: sharedDataWithOrganizationBefore,
    };
  }

  OrganizationDetailScreenArgument.fromMap(Map<String, dynamic> map)
      : organization = map[_kOrganizationKey],
        sharedDataWithOrganizationBefore = map[_kSharedDataBeforeKey];

  @override
  List<Object?> get props => [organization, sharedDataWithOrganizationBefore];
}
