import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/format_relative_time.dart';
import '../../../core/theme/app_colors.dart';
import '../../../core/widgets/panel.dart';
import '../../../core/widgets/status_badge.dart';
import '../../activity/data/activity_entry.dart';
import '../../activity/data/activity_repository.dart';
import '../../clients/data/clients_repository.dart';
import '../../clients/presentation/client_detail_page.dart';
import '../../invoices/data/invoices_repository.dart';
import '../../projects/data/projects_repository.dart';
import '../bloc/dashboard_stats_bloc.dart';
import '../bloc/invoice_summary_bloc.dart';
import '../bloc/recent_activity_bloc.dart';

StatusTone _invoiceStatusTone(String status) => switch (status) {
      'paid' => StatusTone.success,
      'sent' => StatusTone.info,
      'overdue' => StatusTone.danger,
      _ => StatusTone.neutral, // draft, cancelled
    };

String _formatCurrency(double value) => value.toStringAsFixed(2);

/// Matches the web frontend's dashboard (frontend/src/routes/index.tsx)
/// in spirit - independent panels, each backed by its own bloc - though
/// condensed to what fits a phone screen (see AppShellPage's doc comment
/// on why the wider-vs-narrower medium changes more than just the nav
/// shape). No activity_graph/revenue-chart here; the invoice_status and
/// revenue panels are combined into simple numbers rather than the web's
/// pie/bar charts - a real chart is a reasonable follow-up, not
/// attempted in this pass.
class DashboardPage extends StatelessWidget {
  const DashboardPage({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiBlocProvider(
      providers: [
        BlocProvider(
          create: (context) => DashboardStatsBloc(
            clientsRepository: context.read<ClientsRepository>(),
            projectsRepository: context.read<ProjectsRepository>(),
          )..add(const DashboardStatsRequested()),
        ),
        BlocProvider(
          create: (context) => InvoiceSummaryBloc(
            invoicesRepository: context.read<InvoicesRepository>(),
          )..add(const InvoiceSummaryRequested()),
        ),
        BlocProvider(
          create: (context) => RecentActivityBloc(
            activityRepository: context.read<ActivityRepository>(),
          )..add(const RecentActivityRequested()),
        ),
      ],
      child: const _DashboardView(),
    );
  }
}

class _DashboardView extends StatelessWidget {
  const _DashboardView();

  @override
  Widget build(BuildContext context) {
    return RefreshIndicator(
      onRefresh: () async {
        context.read<DashboardStatsBloc>().add(const DashboardStatsRequested());
        context.read<InvoiceSummaryBloc>().add(const InvoiceSummaryRequested());
        context.read<RecentActivityBloc>().add(const RecentActivityRequested());
      },
      child: ListView(
        padding: const EdgeInsets.all(16),
        children: const [
          Panel(title: 'status', child: _StatsPanel()),
          SizedBox(height: 16),
          Panel(title: 'invoice_status', child: _InvoiceSummaryPanel()),
          SizedBox(height: 16),
          Panel(title: 'recent_activity', child: _RecentActivityPanel()),
        ],
      ),
    );
  }
}

class _StatsPanel extends StatelessWidget {
  const _StatsPanel();

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<DashboardStatsBloc, DashboardStatsState>(
      builder: (context, state) {
        if (state.status == DashboardStatsStatus.loading) {
          return const _PanelLoading();
        }
        if (state.status == DashboardStatsStatus.failure) {
          return _PanelError(
            message: state.errorMessage ?? 'Something went wrong.',
            onRetry: () => context.read<DashboardStatsBloc>().add(const DashboardStatsRequested()),
          );
        }

        return Column(
          children: [
            _StatRow(label: 'clients', value: '${state.clientsCount}'),
            _StatRow(label: 'projects', value: '${state.projectsCount}'),
          ],
        );
      },
    );
  }
}

class _InvoiceSummaryPanel extends StatelessWidget {
  const _InvoiceSummaryPanel();

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<InvoiceSummaryBloc, InvoiceSummaryState>(
      builder: (context, state) {
        if (state.status == InvoiceSummaryStatus.loading) {
          return const _PanelLoading();
        }
        if (state.status == InvoiceSummaryStatus.failure) {
          return _PanelError(
            message: state.errorMessage ?? 'Something went wrong.',
            onRetry: () => context.read<InvoiceSummaryBloc>().add(const InvoiceSummaryRequested()),
          );
        }

        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            for (final entry in state.statusCounts.entries)
              Padding(
                padding: const EdgeInsets.symmetric(vertical: 4),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    StatusBadge(label: entry.key, tone: _invoiceStatusTone(entry.key)),
                    Text('${entry.value}', style: const TextStyle(color: AppColors.text)),
                  ],
                ),
              ),
            const Divider(height: 20),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Text('revenue (paid)', style: TextStyle(color: AppColors.textDim, fontSize: 12)),
                Text(
                  _formatCurrency(state.totalPaidRevenue),
                  style: const TextStyle(color: AppColors.accent, fontWeight: FontWeight.bold),
                ),
              ],
            ),
          ],
        );
      },
    );
  }
}

class _RecentActivityPanel extends StatelessWidget {
  const _RecentActivityPanel();

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<RecentActivityBloc, RecentActivityState>(
      builder: (context, state) {
        if (state.status == RecentActivityStatus.loading) {
          return const _PanelLoading();
        }
        if (state.status == RecentActivityStatus.failure) {
          return _PanelError(
            message: state.errorMessage ?? 'Something went wrong.',
            onRetry: () => context.read<RecentActivityBloc>().add(const RecentActivityRequested()),
          );
        }
        if (state.entries.isEmpty) {
          return const Text('-- no activity yet --', style: TextStyle(color: AppColors.textDim));
        }

        final entries = state.entries.take(8).toList();

        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            for (final entry in entries) _ActivityTile(entry: entry),
          ],
        );
      },
    );
  }
}

class _ActivityTile extends StatelessWidget {
  const _ActivityTile({required this.entry});

  final ActivityEntry entry;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      contentPadding: EdgeInsets.zero,
      dense: true,
      title: Text(entry.description, style: const TextStyle(color: AppColors.text, fontSize: 13)),
      trailing: Text(
        formatRelativeTime(entry.createdAt),
        style: const TextStyle(color: AppColors.textDim, fontSize: 11),
      ),
      onTap: entry.linksToClient
          ? () => Navigator.of(context).push(
                MaterialPageRoute(builder: (_) => ClientDetailPage(clientId: entry.subjectId!)),
              )
          : null,
    );
  }
}

class _StatRow extends StatelessWidget {
  const _StatRow({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: const TextStyle(color: AppColors.textDim)),
          StatusBadge(label: value, tone: StatusTone.info),
        ],
      ),
    );
  }
}

class _PanelLoading extends StatelessWidget {
  const _PanelLoading();

  @override
  Widget build(BuildContext context) {
    return const Padding(
      padding: EdgeInsets.symmetric(vertical: 12),
      child: Center(child: CircularProgressIndicator()),
    );
  }
}

class _PanelError extends StatelessWidget {
  const _PanelError({required this.message, required this.onRetry});

  final String message;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(message, style: const TextStyle(color: AppColors.danger, fontSize: 12)),
        const SizedBox(height: 8),
        OutlinedButton(onPressed: onRetry, child: const Text('retry')),
      ],
    );
  }
}
