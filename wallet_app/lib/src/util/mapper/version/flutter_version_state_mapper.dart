import 'package:wallet_core/core.dart';

import '../../../domain/model/update/version_state.dart';
import '../mapper.dart';

class FlutterVersionStateMapper extends Mapper<FlutterVersionState, VersionState> {
  FlutterVersionStateMapper();

  @override
  VersionState map(FlutterVersionState input) => switch (input) {
        FlutterVersionState_Ok() => VersionStateOk(),
        FlutterVersionState_Notify() => VersionStateNotify(),
        FlutterVersionState_Recommend() => VersionStateRecommend(),
        FlutterVersionState_Warn(:final expiresInSeconds) =>
          VersionStateWarn(timeUntilBlocked: Duration(seconds: expiresInSeconds.toInt())),
        FlutterVersionState_Block() => VersionStateBlock(),
      };
}
