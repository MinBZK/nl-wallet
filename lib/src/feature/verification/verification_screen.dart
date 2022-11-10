import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../wallet_constants.dart';
import '../../wallet_routes.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/confirm_action_sheet.dart';
import 'bloc/back_button_visibility_cubit.dart';
import 'bloc/stepper_progress_cubit.dart';
import 'bloc/verification_bloc.dart';
import 'model/verification_request.dart';
import 'page/confirm_data_attributes_page.dart';
import 'page/confirm_verifier_page.dart';
import 'page/verification_declined_page.dart';
import 'page/verification_success_page.dart';
import 'widget/visibility_cubit_back_button.dart';

const _kIndexOfPageWithBackButton = 1;
const _kNrOfPages = 3;

class VerificationScreen extends StatefulWidget {
  final String? restorationId;

  static String getArguments(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return args as String;
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in a (mock) id when opening the VerificationScreen');
    }
  }

  const VerificationScreen({this.restorationId, Key? key}) : super(key: key);

  @override
  State<VerificationScreen> createState() => _VerificationScreenState();
}

class _VerificationScreenState extends State<VerificationScreen> with RestorationMixin {
  final RestorableDouble _currentPage = RestorableDouble(0.0);

  late BackButtonVisibilityCubit _backButtonCubit;
  late StepperProgressCubit _stepperCubit;
  late PageController _pageController;

  @override
  void initState() {
    super.initState();
    _backButtonCubit = BackButtonVisibilityCubit();
    _stepperCubit = StepperProgressCubit(_kNrOfPages);
    _pageController = PageController();
    _pageController.addListener(() {
      if (_pageController.hasClients) {
        _onPageChanged(_pageController.page ?? 0.0);
      }
    });
  }

  @override
  void dispose() {
    _currentPage.dispose();
    _pageController.dispose();
    _backButtonCubit.close();
    _stepperCubit.close();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MultiBlocProvider(
      providers: [
        BlocProvider.value(value: _backButtonCubit),
        BlocProvider.value(value: _stepperCubit),
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

  Widget _buildLoading() => const CenteredLoadingIndicator();

  Widget _buildError(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(AppLocalizations.of(context).verificationScreenGenericError),
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(AppLocalizations.of(context).verificationScreenGenericErrorCta),
          ),
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
          child: _buildPager(context, state),
        ),
      ],
    );
  }

  Widget _buildPager(BuildContext context, VerificationLoadSuccess state) {
    return WillPopScope(
      child: PageView(
        restorationId: 'confirmation_page_view',
        controller: _pageController,
        physics: const NeverScrollableScrollPhysics(),
        children: [
          ConfirmVerifierPage(
            verifier: state.request.verifier,
            onDecline: () => _denyVerificationRequest(context, state.request),
            onAccept: () => _pageController.nextPage(duration: kDefaultAnimationDuration, curve: Curves.easeInOut),
          ),
          ConfirmDataAttributesPage(
            request: state.request,
            onDecline: () => _denyVerificationRequest(context, state.request),
            onAccept: () => _approveVerificationRequest(context, state.request),
          ),
          _buildResultPage(context, state),
        ],
      ),
      onWillPop: () async {
        final bool shouldPop = _pageController.page?.round() != _kIndexOfPageWithBackButton;
        if (!shouldPop) _pageController.previousPage(duration: kDefaultAnimationDuration, curve: Curves.easeInOut);
        return shouldPop;
      },
    );
  }

  Widget _buildResultPage(BuildContext context, VerificationLoadSuccess state) {
    switch (state.status) {
      case VerificationResult.pendingUser:
      case VerificationResult.loading:
        return const CenteredLoadingIndicator();
      case VerificationResult.approved:
        return VerificationSuccessPage(
          verifierShortName: state.request.verifier.shortName,
          onClosePressed: () => _exitVerificationFlow(context),
        );
      case VerificationResult.denied:
        return VerificationDeclinedPage(
          onClosePressed: () => _exitVerificationFlow(context),
        );
    }
  }

  void _denyVerificationRequest(BuildContext context, VerificationRequest request) async {
    if (request.hasMissingAttributes) {
      context.read<VerificationBloc>().add(const VerificationDenied());
      _goToResultPage(context, animate: false);
    } else {
      if (await _showDeclineDialog(context, request.verifier.shortName) == true) {
        context.read<VerificationBloc>().add(const VerificationDenied());
        _goToResultPage(context, animate: false);
      }
    }
  }

  Future<bool?> _showDeclineDialog(BuildContext context, String organizationName) {
    final locale = AppLocalizations.of(context);
    return ConfirmActionSheet.show(
      context,
      title: locale.verificationScreenCancelSheetTitle,
      description: locale.verificationScreenCancelSheetDescription(organizationName),
      cancelButtonText: locale.verificationScreenCancelSheetNegativeCta,
      confirmButtonText: locale.verificationScreenCancelSheetPositiveCta,
    );
  }

  void _approveVerificationRequest(BuildContext context, VerificationRequest request) async {
    context.read<VerificationBloc>().add(const VerificationApproved());
    _goToResultPage(context);
  }

  /// Pops until we are no longer in a `verificationRoute`. This is used to exit the flow
  /// because just popping would otherwise potentially result in going one screen back,
  /// since we are influencing the popping behaviour with WillPopScope in [_buildPager].
  void _exitVerificationFlow(BuildContext context) => Navigator.popUntil(context, (Route<dynamic> route) {
        return !route.willHandlePopInternally &&
            route is ModalRoute &&
            route.settings.name != WalletRoutes.verificationRoute;
      });

  void _goToPage(BuildContext context, int page, {bool animate = true}) {
    if (animate) {
      _pageController.animateToPage(
        page,
        duration: kDefaultAnimationDuration,
        curve: Curves.easeInOut,
      );
    } else {
      _pageController.jumpToPage(page);
    }
  }

  void _goToResultPage(BuildContext context, {bool animate = true}) =>
      _goToPage(context, _kNrOfPages - 1, animate: animate);

  /// Notify cubits when the page of the [_pageController] is updated and make sure
  /// this state is persisted (and can be restored) by storing the page in the
  /// [_currentPage]. Indirectly causes the BackButton and Stepper UI to update.
  void _onPageChanged(double page) {
    _currentPage.value = page;
    _backButtonCubit.showBackButton(page.round() == _kIndexOfPageWithBackButton);
    _stepperCubit.setPage(page);
  }

  @override
  String? get restorationId => widget.restorationId;

  @override
  void restoreState(RestorationBucket? oldBucket, bool initialRestore) {
    registerForRestoration(_currentPage, 'current_page_view_page');
    if (_currentPage.value > 0.0) _onPageChanged(_currentPage.value);
  }
}
