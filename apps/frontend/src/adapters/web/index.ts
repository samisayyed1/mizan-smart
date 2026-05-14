// Web adapter - Browser implementation
// This file re-exports shared modules and platform-specific modules

import type { RunEnv } from "../types";
import { RunEnvs } from "../types";

// Platform constants
export { isDesktop, isWeb, logger } from "./core";

// Re-export types and constants from shared types
export { RunEnvs } from "../types";
export type {
  AddonFile,
  AddonInstallResult,
  AddonManifest,
  AddonUpdateCheckResult,
  AddonUpdateInfo,
  AddonValidationResult,
  AppInfo,
  BackendEnableSyncResult,
  BackendSyncBackgroundEngineResult,
  BackendSyncBootstrapOverwriteCheckResult,
  BackendSyncBootstrapResult,
  BackendSyncCycleResult,
  BackendSyncEngineStatusResult,
  BackendSyncReconcileReadyResult,
  BackendSyncSnapshotUploadResult,
  BackendSyncStateResult,
  EphemeralKeyPair,
  EventCallback,
  ExtractedAddon,
  FunctionPermission,
  ImportRunsRequest,
  InstalledAddon,
  Logger,
  MarketDataProviderSetting,
  Permission,
  PlatformCapabilities,
  PlatformInfo,
  ProviderCapabilities,
  RunEnv,
  UnlistenFn,
  UpdateCheckPayload,
  UpdateCheckResult,
  UpdateThreadRequest,
  UpdateToolResultRequest,
} from "../types";

// Re-export AI types from features/ai-assistant
export type {
  AiChatMessage,
  AiChatModelConfig,
  AiSendMessageRequest,
  AiStreamEvent,
  AiThread,
  AiToolCall,
  AiToolResult,
  AiUsageStats,
  ListThreadsRequest,
  ThreadPage,
} from "@/features/ai-assistant/types";

/**
 * Runtime environment identifier - always "web" for web builds
 */
export const RUN_ENV: RunEnv = RunEnvs.WEB;

// ============================================================================
// Shared domain modules (identical logic for both platforms)
// ============================================================================

// Account Commands
export { createAccount, deleteAccount, getAccounts, updateAccount } from "../shared/accounts";

// Activity Commands
export {
  checkActivitiesImport,
  checkExistingDuplicates,
  createActivity,
  createFixedDeposit,
  createRecurringBuyPlan,
  deleteImportTemplate,
  deleteActivity,
  getImportTemplate,
  getAccountImportMapping,
  linkAccountTemplate,
  linkTransferActivities,
  unlinkTransferActivities,
  getActivities,
  importActivities,
  listImportTemplates,
  previewImportAssets,
  saveAccountImportMapping,
  saveImportTemplate,
  saveActivities,
  searchActivities,
  updateActivity,
} from "../shared/activities";
export { analyzeCsvImport, parseCsv } from "./activities";

// Goal Commands
export {
  createGoal,
  deleteGoal,
  deleteGoalPlan,
  getGoal,
  getGoalFunding,
  getGoalPlan,
  getGoals,
  getRetirementOverview,
  getSaveUpOverview,
  previewSaveUpOverview,
  refreshAllGoalSummaries,
  refreshGoalSummary,
  saveGoalFunding,
  saveGoalPlan,
  updateGoal,
} from "../shared/goals";

// Secrets Commands
export { deleteSecret, getSecret, setSecret } from "../shared/secrets";

// Taxonomy Commands
export {
  assignAssetToCategory,
  createCategory,
  createTaxonomy,
  deleteCategory,
  deleteTaxonomy,
  exportTaxonomyJson,
  getAssetTaxonomyAssignments,
  getMigrationStatus,
  getTaxonomies,
  getTaxonomy,
  importTaxonomyJson,
  migrateLegacyClassifications,
  moveCategory,
  removeAssetTaxonomyAssignment,
  updateCategory,
  updateTaxonomy,
} from "../shared/taxonomies";

// Portfolio Commands
export {
  calculateAccountsSimplePerformance,
  calculatePerformanceHistory,
  calculatePerformanceSummary,
  checkHoldingsImport,
  deleteSnapshot,
  getAssetHoldings,
  getHistoricalValuations,
  getEstimatedHistoricalValuation,
  getHolding,
  getHoldings,
  getHoldingsByAllocation,
  getIncomeSummary,
  getLatestValuations,
  getPortfolioAllocations,
  getSnapshotByDate,
  getSnapshots,
  importHoldingsCsv,
  recalculatePortfolio,
  saveManualHoldings,
  updatePortfolio,
} from "../shared/portfolio";

// Market Data Commands
export {
  checkQuotesImport,
  createAsset,
  deleteAsset,
  deleteQuote,
  fetchYahooDividends,
  getAssetProfile,
  getAssets,
  getExchanges,
  getLatestQuotes,
  getMarketDataProviders,
  getMarketDataProviderSettings,
  getQuoteHistory,
  importManualQuotes,
  resolveSymbolQuote,
  searchTicker,
  syncHistoryQuotes,
  syncMarketData,
  updateAssetProfile,
  updateMarketDataProviderSettings,
  updateQuote,
  updateQuoteMode,
} from "../shared/market-data";

// Custom Provider Commands
export {
  getCustomProviders,
  createCustomProvider,
  updateCustomProvider,
  deleteCustomProvider,
  testCustomProviderSource,
} from "../shared/custom-provider";

// Contribution Limits Commands
export {
  calculateDepositsForLimit,
  createContributionLimit,
  deleteContributionLimit,
  getContributionLimit,
  updateContributionLimit,
} from "../shared/contribution-limits";

// Exchange Rates Commands
export {
  addExchangeRate,
  deleteExchangeRate,
  getExchangeRates,
  updateExchangeRate,
} from "../shared/exchange-rates";

// Alternative Assets Commands
export {
  createAlternativeAsset,
  deleteAlternativeAsset,
  getAlternativeHoldings,
  getNetWorth,
  getNetWorthHistory,
  linkLiability,
  unlinkLiability,
  updateAlternativeAssetMetadata,
  updateAlternativeAssetValuation,
} from "../shared/alternative-assets";

// Universal Add Asset (mizan-smart Phase 1 P5)
export { createUniversalAsset } from "../shared/universal-assets";
export type {
  CreateUniversalAssetResponse,
  UniversalAssetClassification,
  UniversalAssetCommon,
  UniversalAssetCreateRequest,
  FixedIncomeSubtype,
  CommodityRequestType,
  LiabilityRequestType,
} from "../shared/universal-assets";

// Manual valuation grid (mizan-smart Phase 1 P6)
export {
  bulkUpdateValuations,
  getManualValuationHistory,
  listManualValuationAssets,
} from "../shared/manual-valuations";
export type {
  BulkUpdateValuationsRequest,
  BulkUpdateValuationsResult,
  ManualValuationAsset,
  ManualValuationHistoryRow,
  ManualValuationStaleness,
  ManualValuationUpdateRow,
  RowValidationError,
} from "../shared/manual-valuations";

// Wealth Inbox (mizan-smart Phase 1 P9)
export { listWealthInboxItems } from "../shared/inbox";
export type { InboxItem, InboxItemType, InboxSeverity, InboxStatus } from "../shared/inbox";

// Document Vault (mizan-smart Phase 2 P10)
export {
  deleteDocument,
  getDocumentMetadata,
  listDocuments,
  readDocumentBytes,
  uploadDocument,
} from "../shared/documents";
export type {
  DocumentFileMetadata,
  DocumentMetadata,
  DocumentRecord,
  DocumentStatus,
  UploadDocumentRequest,
} from "../shared/documents";
export {
  cancelDocumentJob,
  enqueueDocumentJob,
  getDocumentParserCapabilities,
  getParsedDocument,
  listDocumentJobs,
  retryDocumentJob,
  runNextDocumentJob,
} from "../shared/document-jobs";
export type {
  DocumentJobStatus,
  DocumentJobType,
  DocumentBoundingBox,
  DocumentProcessingJob,
  DocumentParserCapabilities,
  EnqueueDocumentJobRequest,
  ParsedDocument,
  ParsedDocumentPage,
  ParsedTable,
  ParsedTableCell,
  ParsedTextBlock,
  RunDocumentJobResult,
} from "../shared/document-jobs";
export {
  approveExtractedFact,
  createExtractedFact,
  deferExtractedFact,
  getSourceCitation,
  linkExtractedFactToEntity,
  listPendingExtractedFacts,
  rejectExtractedFact,
  updateExtractedFactBeforeApproval,
} from "../shared/extracted-facts";
export type {
  CreateExtractedFactRequest,
  CreateExtractedFactResult,
  DeferExtractedFactRequest,
  ExtractedFact,
  ExtractedFactEntityLink,
  ExtractedFactLinkEntityType,
  ExtractedFactStatus,
  ExtractionMethod,
  LinkExtractedFactRequest,
  ReviewExtractedFactRequest,
  SourceCitation,
  SourceCitationType,
  UpdateExtractedFactRequest,
} from "../shared/extracted-facts";
export { getDataLineage } from "../shared/data-lineage";
export {
  acceptReconciliationAdjustment,
  getReconciliationRun,
  ignoreReconciliationMatch,
  manualReconciliationMatch,
  reconcileAccount,
  reconcileDocumentFacts,
  reconcileImportPreview,
} from "../shared/reconciliation";
export {
  addCapitalCall,
  addPrivateDistribution,
  addPrivateInvestmentValuation,
  deletePrivateInvestment,
  getPrivateInvestment,
  getPrivateInvestmentSummary,
  updateCapitalCallStatus,
  upsertPrivateInvestment,
} from "../shared/private-investments";
export type {
  DataLineageEntityType,
  DataLineageFxRate,
  DataLineageInputRow,
  DataLineageMetricType,
  DataLineageResponse,
  DataLineageSourceCitation,
  DataLineageSourceDocument,
  GetDataLineageRequest,
} from "../shared/data-lineage";
export type {
  AcceptReconciliationAdjustmentRequest,
  AcceptReconciliationAdjustmentResult,
  IgnoreReconciliationMatchRequest,
  JsonValue,
  ManualReconciliationMatchRequest,
  ReconcileAccountRequest,
  ReconcileDocumentFactsRequest,
  ReconcileImportPreviewRequest,
  ReconciliationInputItem,
  ReconciliationItem,
  ReconciliationItemStatus,
  ReconciliationMatch,
  ReconciliationMatchStatus,
  ReconciliationRun,
  ReconciliationRunDetail,
  ReconciliationRunStatus,
  ReconciliationScopeType,
  ReconciliationSourceSide,
} from "../shared/reconciliation";
export type {
  CapitalCall,
  CapitalCallStatus,
  CreateCapitalCallRequest,
  CreatePrivateDistributionRequest,
  CreatePrivateInvestmentValuationRequest,
  PrivateDistribution,
  PrivateInvestment,
  PrivateInvestmentSummary,
  PrivateInvestmentValuation,
  UpdateCapitalCallStatusRequest,
  UpsertPrivateInvestmentRequest,
} from "../shared/private-investments";

// Connect Commands (Broker + Device Sync + Auth)
export {
  approvePairing,
  approvePairingOverwrite,
  beginPairingConfirm,
  cancelPairing,
  cancelPairingFlow,
  claimPairing,
  clearDeviceSyncData,
  clearSyncSession,
  completePairing,
  completePairingWithTransfer,
  confirmPairing,
  confirmPairingWithBootstrap,
  createPairing,
  getPairingFlowState,
  deleteDevice,
  deviceSyncBootstrapOverwriteCheck,
  deviceSyncCancelSnapshotUpload,
  deviceSyncGenerateSnapshotNow,
  deviceSyncReconcileReadyState,
  deviceSyncStartBackgroundEngine,
  deviceSyncStopBackgroundEngine,
  enableDeviceSync,
  getBrokerSyncStates,
  getDevice,
  getDeviceSyncState,
  getImportRuns,
  getPairingSourceStatus,
  getPairing,
  getPairingMessages,
  getPlatforms,
  getSubscriptionPlans,
  getSubscriptionPlansPublic,
  getSyncedAccounts,
  getSyncEngineStatus,
  getUserInfo,
  createBrokerLoginPortal,
  deleteBrokerConnection,
  listBrokerAccounts,
  listBrokerConnections,
  listDevices,
  reinitializeDeviceSync,
  resetTeamSync,
  restoreSyncSession,
  revokeDevice,
  storeSyncSession,
  syncBootstrapSnapshotIfNeeded,
  syncBrokerData,
  syncTriggerCycle,
  updateDevice,
} from "../shared/connect";

// AI Providers Commands
export {
  getAiProviders,
  listAiModels,
  setDefaultAiProvider,
  updateAiProviderSettings,
} from "../shared/ai-providers";

// AI Threads Commands
export {
  addAiThreadTag,
  deleteAiThread,
  getAiThread,
  getAiThreadMessages,
  getAiThreadTags,
  listAiThreads,
  removeAiThreadTag,
  updateAiThread,
  updateToolResult,
} from "../shared/ai-threads";

// Health Center Commands
export {
  dismissHealthIssue,
  executeHealthFix,
  getDismissedHealthIssues,
  getHealthConfig,
  getHealthStatus,
  restoreHealthIssue,
  runHealthChecks,
  updateHealthConfig,
} from "../shared/health";

// ============================================================================
// Platform-specific modules (different implementations for web vs desktop)
// ============================================================================

// AI Streaming (web-specific HTTP fetch implementation)
export { streamAiChat } from "./ai-streaming";

// Event Listeners (web-specific SSE implementation)
export {
  listenBrokerSyncComplete,
  listenBrokerSyncError,
  listenBrokerSyncStart,
  listenDatabaseRestored,
  listenDeepLink,
  listenFileDrop,
  listenFileDropCancelled,
  listenFileDropHover,
  listenMarketSyncComplete,
  listenMarketSyncError,
  listenMarketSyncStart,
  listenNavigateToRoute,
  listenPortfolioUpdateComplete,
  listenPortfolioUpdateError,
  listenPortfolioUpdateStart,
} from "./events";

// File Dialogs (web-specific implementations)
export {
  openCsvFileDialog,
  openDatabaseFileDialog,
  openFileSaveDialog,
  openFolderDialog,
  openUrlInBrowser,
} from "./files";

// Settings Commands (web-specific API for backups and updates)
export {
  backupDatabase,
  backupDatabaseToPath,
  backupDatabaseToPathEncrypted,
  checkForUpdates,
  getAppInfo,
  getPlatform,
  getSettings,
  installUpdate,
  isAutoUpdateCheckEnabled,
  restoreDatabase,
  updateSettings,
} from "./settings";

// Addon Commands (web-specific implementations)
export {
  checkAddonUpdate,
  checkAllAddonUpdates,
  clearAddonStaging,
  downloadAddonForReview,
  extractAddon,
  extractAddonZip,
  fetchAddonStoreListings,
  getAddonRatings,
  getEnabledAddons,
  getEnabledAddonsOnStartup,
  getInstalledAddons,
  installAddon,
  installAddonFile,
  installAddonZip,
  installFromStaging,
  listInstalledAddons,
  loadAddon,
  loadAddonForRuntime,
  submitAddonRating,
  toggleAddon,
  uninstallAddon,
  updateAddon,
} from "./addons";

// FIRE Planner (desktop-only — stubs throw at runtime)
export {
  calculateRetirementProjection,
  runRetirementDecisionSensitivityMap,
  runRetirementMonteCarlo,
  runRetirementScenarioAnalysis,
  runRetirementSorr,
  runRetirementStressTests,
} from "./fire-planner";

// Crypto Commands (web stubs - not available in web mode)
export {
  syncComputeSas,
  syncComputeSharedSecret,
  syncDecrypt,
  syncDeriveDek,
  syncDeriveSessionKey,
  syncEncrypt,
  syncGenerateDeviceId,
  syncGenerateKeypair,
  syncGeneratePairingCode,
  syncGenerateRootKey,
  syncHashPairingCode,
  syncHmacSha256,
} from "./crypto";
