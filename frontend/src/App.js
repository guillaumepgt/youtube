import React, { useState, useEffect } from 'react';
import axios from 'axios';
import VideoGrid from './components/VideoGrid';

class ErrorBoundary extends React.Component {
	state = { error: null };

	static getDerivedStateFromError(error) {
		return { error: error.message };
	}

	render() {
		if (this.state.error) {
			return <div className="error">Erreur de rendu: {this.state.error}</div>;
		}
		return this.props.children;
	}
}

function App() {
	const [videos, setVideos] = useState([]);
	const [isAuthenticated, setIsAuthenticated] = useState(false);
	const [loading, setLoading] = useState(false);
	const [error, setError] = useState(null);

	useEffect(() => {
		console.log('App mounted, checking authentication');
		const urlParams = new URLSearchParams(window.location.search);

		// Vérifier si on a une erreur d'authentification
		if (urlParams.has('error')) {
			setError("Erreur lors de l'authentification avec Google");
			window.history.replaceState({}, document.title, '/');
			return;
		}

		// Si on revient de Google OAuth avec les tokens
		if (urlParams.has('access_token')) {
			const access_token = urlParams.get('access_token');
			const refresh_token = urlParams.get('refresh_token');
			const expires_in = urlParams.get('expires_in');

			console.log('Tokens received from callback:', {
				access_token: access_token ? 'present' : 'missing',
				refresh_token: refresh_token ? 'present' : 'missing',
				expires_in
			});

			// Sauvegarder les tokens
			if (access_token) {
				localStorage.setItem('access_token', access_token);
				console.log('Access token saved');
			}
			if (refresh_token && refresh_token !== '') {
				localStorage.setItem('refresh_token', refresh_token);
				console.log('Refresh token saved');
			}
			if (expires_in) {
				localStorage.setItem('expires_in', expires_in);
				console.log('Expires_in saved:', expires_in);
			}

			setIsAuthenticated(true);
			window.history.replaceState({}, document.title, '/');

			// Charger les vidéos après un court délai pour s'assurer que tout est sauvegardé
			setTimeout(() => fetchVideos(), 100);
		} else {
			// Vérifier si on a déjà un token
			const savedToken = localStorage.getItem('access_token');
			if (savedToken) {
				console.log('Found saved access token');
				setIsAuthenticated(true);
				fetchVideos();
			}
		}
	}, []);

	// Effet pour débugger le localStorage
	useEffect(() => {
		console.log('Authentication state changed:', isAuthenticated);
		console.log('LocalStorage contents:', {
			access_token: localStorage.getItem('access_token') ? 'present' : 'missing',
			refresh_token: localStorage.getItem('refresh_token') ? 'present' : 'missing',
			expires_in: localStorage.getItem('expires_in')
		});
	}, [isAuthenticated]);

	const handleLogin = () => {
		console.log('Redirecting to login...');
		window.location.href = 'http://localhost:8080/login';
	};

	const fetchVideos = async (retries = 3, delay = 2000) => {
		setLoading(true);
		setError(null);

		const access_token = localStorage.getItem('access_token');
		const refresh_token = localStorage.getItem('refresh_token');
		const expires_in = localStorage.getItem('expires_in');

		console.log('Fetching videos with:', {
			access_token: access_token ? 'present' : 'missing',
			refresh_token: refresh_token ? 'present' : 'missing',
			expires_in
		});

		if (!access_token) {
			console.error('No access_token found');
			setError("Erreur: aucun token d'accès disponible");
			setLoading(false);
			return;
		}

		for (let i = 0; i < retries; i++) {
			try {
				const headers = {
					'Authorization': `Bearer ${access_token}`
				};

				if (refresh_token && refresh_token !== '') {
					headers['refresh_token'] = refresh_token;
				}
				if (expires_in) {
					headers['expires_in'] = expires_in;
				}

				console.log(`Attempt ${i + 1}: Sending request to /subscriptions/videos`);

				const response = await axios.get('http://localhost:8080/subscriptions/videos', {
					headers,
					timeout: 30000,
				});

				console.log('Videos response:', response.data);

				if (Array.isArray(response.data)) {
					setVideos(response.data);
					setError(null);
				} else if (response.data.message) {
					setError(response.data.message);
				} else {
					console.warn('Unexpected response format:', response.data);
					setError('Réponse inattendue du serveur');
				}

				setLoading(false);
				return;
			} catch (err) {
				console.error(`Attempt ${i + 1} failed:`, err.message);

				if (err.response && err.response.status === 401) {
					console.log('Token expiré, reconnexion nécessaire');
					handleLogout();
					setError("Session expirée, veuillez vous reconnecter");
					setLoading(false);
					return;
				}

				if (i < retries - 1) {
					console.log(`Retrying in ${delay}ms...`);
					await new Promise((resolve) => setTimeout(resolve, delay));
				} else {
					setError(`Erreur lors de la récupération des vidéos: ${err.message}`);
					setLoading(false);
				}
			}
		}
	};

	const handleLogout = () => {
		console.log('Logging out...');
		localStorage.removeItem('access_token');
		localStorage.removeItem('refresh_token');
		localStorage.removeItem('expires_in');
		setIsAuthenticated(false);
		setVideos([]);
		setError(null);
	};

	if (!isAuthenticated) {
		return (
			<div style={{ textAlign: 'center', padding: '2rem' }}>
				<header>
					<h1>Bienvenue sur ton app YouTube Subs</h1>
					<button
						onClick={handleLogin}
						style={{
							padding: '10px 20px',
							fontSize: '16px',
							backgroundColor: '#007bff',
							color: 'white',
							border: 'none',
							borderRadius: '5px',
							cursor: 'pointer'
						}}
					>
						Se connecter avec Google
					</button>
					{error && <p style={{ color: 'red', marginTop: '1rem' }}>{error}</p>}
				</header>
			</div>
		);
	}

	return (
		<ErrorBoundary>
			<div>
				<header style={{
					backgroundColor: '#282c34',
					padding: '1rem',
					color: 'white',
					display: 'flex',
					justifyContent: 'space-between',
					alignItems: 'center'
				}}>
					<h1>Vidéos des abonnements</h1>
					<div>
						<button
							onClick={() => fetchVideos()}
							disabled={loading}
							style={{
								padding: '8px 16px',
								marginRight: '10px',
								backgroundColor: loading ? '#ccc' : '#28a745',
								color: 'white',
								border: 'none',
								borderRadius: '5px',
								cursor: loading ? 'not-allowed' : 'pointer'
							}}
						>
							{loading ? 'Chargement...' : 'Rafraîchir'}
						</button>
						<button
							onClick={handleLogout}
							style={{
								padding: '8px 16px',
								backgroundColor: '#dc3545',
								color: 'white',
								border: 'none',
								borderRadius: '5px',
								cursor: 'pointer'
							}}
						>
							Déconnexion
						</button>
					</div>
				</header>
				{loading && <p style={{ textAlign: 'center', padding: '1rem' }}>Chargement des vidéos...</p>}
				{error && <p style={{ color: 'red', textAlign: 'center', padding: '1rem' }}>{error}</p>}
				<VideoGrid videos={videos} />
			</div>
		</ErrorBoundary>
	);
}

export default App;