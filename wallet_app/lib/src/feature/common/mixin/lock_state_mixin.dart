import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/wallet/observe_wallet_locked_usecase.dart';

mixin LockStateMixin<T extends StatefulWidget> on State<T> {
  StreamSubscription? _lockSubscription;

  @override
  void initState() {
    super.initState();
    _lockSubscription = context
        .read<ObserveWalletLockedUseCase>()
        .invoke()
        .skip(1 /* skip initial value */)
        .distinct(/* only track changes */)
        .listen(_onLockChanged);
  }

  @override
  void dispose() {
    _lockSubscription?.cancel();
    super.dispose();
  }

  Future<void> _onLockChanged(bool locked) async {
    if (locked) {
      await onLock();
    } else {
      await onUnlock();
    }
  }

  FutureOr<void> onLock();

  FutureOr<void> onUnlock();
}
