import React, { useContext } from 'react';
import { Navigate } from 'react-router';
import { Route, Routes } from 'react-router-dom';

import { AppPropsContext } from '@stump/client';

import { AppLayout } from './AppLayout';

// TODO: finish import() for lazy loading...
import FourOhFour from './pages/FourOhFour';
import LoginOrClaim from './pages/LoginOrClaim';
import OnBoarding from './pages/OnBoarding';

const Home = React.lazy(() => import('./pages/Home'));
const LibraryOverview = React.lazy(() => import('./pages/library/LibraryOverview'));
const LibraryFileExplorer = React.lazy(() => import('./pages/library/LibraryFileExplorer'));
const SeriesOverview = React.lazy(() => import('./pages/SeriesOverview'));
const BookOverview = React.lazy(() => import('./pages/book/BookOverview'));
const SettingsLayout = React.lazy(() => import('./components/settings/SettingsLayout'));
const GeneralSettings = React.lazy(() => import('./pages/settings/GeneralSettings'));
function OnBoardingRouter() {
	return (
		<Routes>
			<Route path="/" element={<OnBoarding />} />
		</Routes>
	);
}

export function AppRouter() {
	const appProps = useContext(AppPropsContext);

	if (!appProps?.baseUrl) {
		if (appProps?.platform === 'browser') {
			throw new Error('Base URL is not set');
		}

		return <OnBoardingRouter />;
	}

	return (
		<Routes>
			<Route path="/" element={<AppLayout />}>
				<Route path="" element={<Home />} />

				<Route path="libraries/:id" element={<LibraryOverview />} />
				<Route path="libraries/:id/explorer" element={<LibraryFileExplorer />} />
				<Route path="series/:id" element={<SeriesOverview />} />
				<Route path="books/:id" element={<BookOverview />} />

				<Route path="settings" element={<SettingsLayout />}>
					<Route path="" element={<Navigate to="/settings/general" replace />} />
					<Route path="general" element={<GeneralSettings />} />
				</Route>
			</Route>

			<Route path="/auth" element={<LoginOrClaim />} />
			<Route path="*" element={<FourOhFour />} />

			{/* 
		<Route path="settings" element={<Settings />}>
			<Route path="" element={<Navigate to="/settings/general" replace={true} />} />
			<Route path="general" element={<GeneralSettings />} />
			<Route path="users" element={<UserSettings />} />
			<Route path="server" element={<ServerSettings />} />
			<Route path="jobs" element={<JobSettingsTab />} />
		</Route>
		<Route path="libraries/:id/explorer" element={<LibraryFileExplorer />} />
		<Route path="books/:id/pages/:page" element={<ReadBook />} />
		<Route path="epub/:id" element={<ReadEpub />} />
		<Route path="epub/:id/loc/:loc" element={<ReadEpub />} />
	</Route>

	<Route path="/auth" element={<BaseLayout />}>
		<Route path="login" element={<Login />} />
	</Route>
	<Route path="*" element={<FourOhFour />} /> */}
		</Routes>
	);
}
