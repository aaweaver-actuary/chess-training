export * from './services/PgnImportService.js';
export * from './services/CommandPaletteService.js';
export * from './services/ImportPlanner.js';
export * from './controllers/OpeningReviewController.js';
export * from './controllers/SessionController.js';
export * from './viewModels/DashboardViewModel.js';

export const applicationServices = {
  pgnImport: Symbol.for('application:PgnImportService'),
  commandPalette: Symbol.for('application:CommandPaletteService'),
  importPlanner: Symbol.for('application:ImportPlanner'),
} as const;

export type ApplicationServiceTokens = typeof applicationServices;

export const applicationControllers = {
  openingReview: Symbol.for('application:OpeningReviewController'),
  session: Symbol.for('application:SessionController'),
} as const;

export type ApplicationControllerTokens = typeof applicationControllers;

export const applicationViewModels = {
  dashboard: Symbol.for('application:DashboardViewModel'),
} as const;

export type ApplicationViewModelTokens = typeof applicationViewModels;
