import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../wallet_constants.dart';
import '../bloc/back_button_visibility_cubit.dart';

/// Back button that is hidden/shown based on the state of
/// the nearest [BackButtonVisibilityCubit].
class VisibilityCubitBackButton extends StatelessWidget {
  const VisibilityCubitBackButton({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<BackButtonVisibilityCubit, bool>(
      builder: (context, visible) {
        return AnimatedOpacity(
          duration: kDefaultAnimationDuration,
          opacity: visible ? 1 : 0,
          child: const BackButton(),
        );
      },
    );
  }
}
