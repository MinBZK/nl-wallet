import 'dart:async';

import 'package:after_layout/after_layout.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';

import '../../../domain/usecase/notification/observe_os_notifications_usecase.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/list/list_item.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/wallet_app_bar.dart';

/// A screen for displaying debug information related to Notifications.
/// ONLY FOR DEBUG PURPOSES
class ScheduledNotificationsScreen extends StatefulWidget {
  const ScheduledNotificationsScreen({super.key});

  @override
  State<ScheduledNotificationsScreen> createState() => _ScheduledNotificationsScreenState();

  static void show(BuildContext context) =>
      Navigator.of(context).push(MaterialPageRoute(builder: (c) => const ScheduledNotificationsScreen()));
}

class _ScheduledNotificationsScreenState extends State<ScheduledNotificationsScreen>
    with AfterLayoutMixin<ScheduledNotificationsScreen> {
  final FlutterLocalNotificationsPlugin _plugin = FlutterLocalNotificationsPlugin();

  List<PendingNotificationRequest> _pendingNotifications = [];
  List<ActiveNotification> _activeNotifications = [];

  @override
  FutureOr<void> afterFirstLayout(BuildContext context) => _loadNotifications();

  Future<void> _loadNotifications() async {
    _pendingNotifications = await _plugin.pendingNotificationRequests();
    _activeNotifications = await _plugin.getActiveNotifications();
    setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    return DefaultTabController(
      length: 3,
      child: Scaffold(
        appBar: WalletAppBar(
          fadeInTitleOnScroll: false,
          title: const Text('Notifications'),
          actions: [IconButton(onPressed: _loadNotifications, icon: const Icon(Icons.refresh_outlined))],
          bottom: const TabBar(
            tabs: [
              Tab(text: 'Core'),
              Tab(text: 'Pending'),
              Tab(text: 'Active'),
            ],
          ),
        ),
        body: TabBarView(
          children: [
            _buildCoreNotificationList(),
            _buildPendingNotificationsList(),
            _buildActiveNotificationsList(),
          ],
        ),
      ),
    );
  }

  Widget _buildCoreNotificationList() {
    return StreamBuilder(
      stream: context.read<ObserveOsNotificationsUseCase>().invoke(respectUserSetting: false),
      builder: (context, state) {
        final notifications = state.data ?? [];
        if (notifications.isEmpty) {
          return const Center(
            child: TitleText(
              'No core notifications',
              textAlign: .center,
            ),
          );
        }
        return ListView.builder(
          itemCount: notifications.length + 1,
          padding: const EdgeInsets.symmetric(vertical: 16),
          itemBuilder: (context, index) {
            if (index == 0) return _buildCoreHeader();
            final notification = notifications[index - 1];
            return ListItem.horizontal(
              label: Text(notification.title),
              subtitle: Column(
                crossAxisAlignment: .start,
                children: [
                  Text(notification.body),
                  const SizedBox(height: 4),
                  Text(
                    'id: ${notification.id}',
                    style: context.textTheme.bodySmall,
                  ),
                  Text(
                    'channel: ${notification.channel}',
                    style: context.textTheme.bodySmall,
                  ),
                  Text(
                    'notifyAt: ${notification.notifyAt}',
                    style: context.textTheme.bodySmall,
                  ),
                ],
              ),
              icon: const Icon(Icons.business_outlined),
            );
          },
        );
      },
    );
  }

  Widget _buildCoreHeader() {
    return ListItem.compact(
      label: const Text('Core Notifications'),
      subtitle: Text(
        'These are the notifications as exposed by the wallet_core. Having them in this list does not mean they are scheduled by the system.',
        style: context.textTheme.bodySmall,
      ),
      icon: const Icon(Icons.help_outline_outlined),
    );
  }

  Widget _buildPendingNotificationsList() {
    if (_pendingNotifications.isEmpty) {
      return const Center(
        child: TitleText(
          'No pending notifications',
          textAlign: .center,
        ),
      );
    }
    return ListView.builder(
      itemCount: _pendingNotifications.length + 1,
      padding: const EdgeInsets.symmetric(vertical: 16),
      itemBuilder: (context, index) {
        if (index == 0) return _buildPendingHeader();
        final notification = _pendingNotifications[index - 1];
        return ListItem.horizontal(
          label: Text(notification.title ?? 'No title'),
          subtitle: Column(
            crossAxisAlignment: .start,
            children: [
              Text(notification.body ?? 'No body'),
              const SizedBox(height: 4),
              Text(
                'id: ${notification.id}',
                style: context.textTheme.bodySmall,
              ),
              Text(
                'payload: ${notification.payload}',
                style: context.textTheme.bodySmall,
              ),
            ],
          ),
          icon: const Icon(Icons.schedule),
        );
      },
    );
  }

  Widget _buildPendingHeader() {
    return ListItem.compact(
      label: const Text('Pending Notifications'),
      subtitle: Text(
        'These are the notifications have been scheduled by the system, they will be shown even when the app is in the background.',
        style: context.textTheme.bodySmall,
      ),
      icon: const Icon(Icons.help_outline_outlined),
    );
  }

  Widget _buildActiveNotificationsList() {
    if (_activeNotifications.isEmpty) {
      return const Center(
        child: TitleText(
          'No active notifications',
          textAlign: .center,
        ),
      );
    }
    return ListView.builder(
      itemCount: _activeNotifications.length + 1,
      padding: const EdgeInsets.symmetric(vertical: 16),
      itemBuilder: (context, index) {
        if (index == 0) return _buildActiveHeader();
        final notification = _activeNotifications[index - 1];
        return ListItem.horizontal(
          label: Text(notification.title ?? 'No title'),
          subtitle: Column(
            crossAxisAlignment: .start,
            children: [
              Text(notification.body ?? 'No body'),
              const SizedBox(height: 4),
              Text('id: ${notification.id}', style: context.textTheme.bodySmall),
              Text('payload: ${notification.payload}', style: context.textTheme.bodySmall),
            ],
          ),
          icon: const Icon(Icons.notifications_active),
        );
      },
    );
  }

  Widget _buildActiveHeader() {
    return ListItem.compact(
      label: const Text('Active Notifications'),
      subtitle: Text(
        'These are the notifications that have been shown to the user, and are currently visible in the notification tray.',
        style: context.textTheme.bodySmall,
      ),
      icon: const Icon(Icons.help_outline_outlined),
    );
  }
}
