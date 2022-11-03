import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../wallet_constants.dart';
import '../../wallet_routes.dart';
import 'bloc/back_button_visibility_cubit.dart';
import 'bloc/stepper_progress_cubit.dart';
import 'bloc/verification_bloc.dart';
import 'model/verification_request.dart';
import 'page/confirm_data_attributes_page.dart';
import 'page/confirm_verifier_page.dart';
import 'page/verification_success_page.dart';
import 'widget/visibility_cubit_back_button.dart';

const _kIndexOfPageWithBackButton = 1;
const _kNrOfPages = 3;

class VerificationScreen extends StatelessWidget {
  static String getArguments(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return args as String;
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in a (mock) id when opening the VerificationScreen');
    }
  }

  const VerificationScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MultiBlocProvider(
      providers: [
        BlocProvider(create: (context) => BackButtonVisibilityCubit()),
        BlocProvider(create: (context) => StepperProgressCubit(_kNrOfPages)),
      ],
      child: Scaffold(
        appBar: AppBar(
          title: Text(AppLocalizations.of(context).verificationScreenTitle),
          leading: const VisibilityCubitBackButton(),
          actions: [CloseButton(onPressed: () => _exitVerificationFlow(context))],
        ),
        body: BlocBuilder<VerificationBloc, VerificationState>(
          builder: (context, state) {
            if (state is VerificationInitial) return _buildLoading();
            if (state is VerificationLoadInProgress) return _buildLoading();
            if (state is VerificationLoadFailure) return _buildError(context);
            if (state is VerificationLoadSuccess) return _buildSuccess(context, state);
            throw UnsupportedError('Unknown state: $state');
          },
        ),
      ),
    );
  }

  /// Pops until we are no longer in a `verificationRoute`. This is used to exit the flow
  /// because just popping would otherwise potentially result in going one screen back,
  /// since we are influencing the popping behaviour with WillPopScope in [_buildPager].
  void _exitVerificationFlow(BuildContext context) => Navigator.popUntil(context, (Route<dynamic> route) {
        return !route.willHandlePopInternally &&
            route is ModalRoute &&
            route.settings.name != WalletRoutes.verificationRoute;
      });

  Widget _buildLoading() => const Center(child: CircularProgressIndicator());

  Widget _buildError(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(AppLocalizations.of(context).verificationScreenGenericError),
          TextButton(onPressed: () => Navigator.pop(context), child: const Text('Ga terug')),
        ],
      ),
    );
  }

  Widget _buildSuccess(BuildContext context, VerificationLoadSuccess state) {
    return Column(
      children: [
        BlocBuilder<StepperProgressCubit, double>(
          builder: (context, progress) => LinearProgressIndicator(value: progress),
        ),
        Expanded(
          child: _buildPager(context, state.request),
        ),
      ],
    );
  }

  Widget _buildPager(BuildContext context, VerificationRequest request) {
    final controller = PageController();
    return AnimatedBuilder(
      animation: controller,
      child: WillPopScope(
        child: PageView(
          restorationId: 'confirmation_page_view',
          controller: controller,
          physics: const NeverScrollableScrollPhysics(),
          children: [
            ConfirmVerifierPage(
              verifier: request.verifier,
              onDecline: () => _exitVerificationFlow(context),
              onAccept: () => controller.nextPage(duration: kDefaultAnimationDuration, curve: Curves.easeInOut),
            ),
            ConfirmDataAttributesPage(
              request: request,
              onDecline: () => _exitVerificationFlow(context),
              onAccept: () => controller.nextPage(duration: kDefaultAnimationDuration, curve: Curves.easeInOut),
            ),
            VerificationSuccessPage(
              verifierShortName: request.verifier.shortName,
              onClosePressed: () => _exitVerificationFlow(context),
            ),
          ],
        ),
        onWillPop: () async {
          final bool shouldPop = controller.page?.round() != _kIndexOfPageWithBackButton;
          if (!shouldPop) controller.previousPage(duration: kDefaultAnimationDuration, curve: Curves.easeInOut);
          return shouldPop;
        },
      ),
      builder: (context, snapshot) {
        _updateCubits(context, controller);
        return snapshot!;
      },
    );
  }

  /// Notify any related cubits about state changes of the [PageController].
  /// Causes updates to UI elements like the back button and stepper.
  void _updateCubits(BuildContext context, PageController controller) {
    if (controller.hasClients) {
      context.read<BackButtonVisibilityCubit>().showBackButton(controller.page?.round() == _kIndexOfPageWithBackButton);
      context.read<StepperProgressCubit>().setPage(controller.page ?? 0);
    }
  }
}
