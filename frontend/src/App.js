import React, {useState, useEffect} from 'react';
import {BrowserRouter as Router, Routes, Route} from 'react-router-dom';
import axios from 'axios';
import VideoGrid from './components/VideoGrid';
import './App.css';

class ErrorBoundary extends React.Component {
	state = {error: null};

	static getDerivedStateFromError(error) {
		return {error: error.message};
	}

	render() {
		if (this.state.error) {
			return <div className="error">Erreur de rendu: {this.state.error}</div>;
		}
		return this.props.children;
	}
}

function MainApp() {
	const [videos, setVideos] = useState([]);
	const [isAuthenticated, setIsAuthenticated] = useState(false);
	const [loading, setLoading] = useState(false);
	const [error, setError] = useState(null);
	const [searchInput, setSearchInput] = useState('');

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

	// Fonction de recherche
	const handleSearch = async () => {
		if (!searchInput.trim()) {
			setError('Veuillez entrer un terme de recherche');
			return;
		}

		setLoading(true);
		setError(null);

		try {
			const response = await axios.get(`http://localhost:8080/search/${encodeURIComponent(searchInput)}`);
			console.log('Résultats de recherche:', response.data);

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
		} catch (err) {
			console.error('Erreur lors de la recherche:', err.message);
			setError(`Erreur lors de la recherche: ${err.message}`);
			setLoading(false);
		}
	};

	// Permettre la recherche avec la touche Entrée
	const handleKeyPress = (e) => {
		if (e.key === 'Enter') {
			handleSearch();
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
			<div style={{textAlign: 'center', padding: '2rem'}}>
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
					{error && <p style={{color: 'red', marginTop: '1rem'}}>{error}</p>}
				</header>
			</div>
		);
	}

	return (
		<ErrorBoundary>
			<div className="App">
				<header className="App-header">
					<div className="header-left">
						<div className="menu-icon">
							<svg viewBox="0 0 24 24" width="24" height="24">
								<path fill="#fff" d="M3 6h18v2H3V6m0 5h18v2H3v-2m0 5h18v2H3v-2z"/>
							</svg>
						</div>
						<div className="logo">
							<svg viewBox="0 0 90 20" preserveAspectRatio="xMidYMid meet">
								<g>
									<path fill="#FF0000"
												d="M27.9727 3.12324C27.6435 1.89323 26.6768 0.926623 25.4468 0.597366C23.2197 2.24288e-07 14.285 0 14.285 0C14.285 0 5.35042 2.24288e-07 3.12323 0.597366C1.89323 0.926623 0.926623 1.89323 0.597366 3.12324C2.24288e-07 5.35042 0 10 0 10C0 10 2.24288e-07 14.6496 0.597366 16.8768C0.926623 18.1068 1.89323 19.0734 3.12323 19.4026C5.35042 20 14.285 20 14.285 20C14.285 20 23.2197 20 25.4468 19.4026C26.6768 19.0734 27.6435 18.1068 27.9727 16.8768C28.5701 14.6496 28.5701 10 28.5701 10C28.5701 10 28.5677 5.35042 27.9727 3.12324Z"/>
									<path fill="#FFFFFF" d="M11.4253 14.2854L18.8477 10.0004L11.4253 5.71533V14.2854Z"/>
								</g>
								<g>
									<path fill="#FFFFFF"
												d="M34.6024 13.0036L31.3945 1.41846H34.1932L35.3174 6.6701C35.6043 7.96361 35.8136 9.06662 35.95 9.97913H36.0323C36.1264 9.32532 36.3381 8.22937 36.665 6.68892L37.8291 1.41846H40.6278L37.3799 13.0036V18.561H34.6001V13.0036H34.6024Z"/>
									<path fill="#FFFFFF"
												d="M41.4697 18.1937C40.9053 17.8127 40.5031 17.22 40.2632 16.4157C40.0257 15.6114 39.9058 14.5437 39.9058 13.2078V11.3898C39.9058 10.0422 40.0422 8.95805 40.315 8.14196C40.5878 7.32588 41.0135 6.72851 41.592 6.35457C42.1706 5.98063 42.9302 5.79248 43.871 5.79248C44.7976 5.79248 45.5384 5.98298 46.0981 6.36398C46.6555 6.74497 47.0508 7.34234 47.2931 8.15137C47.5331 8.96275 47.6508 10.0422 47.6508 11.3898V13.2078C47.6508 14.5437 47.5331 15.6161 47.2908 16.4251C47.0508 17.2365 46.644 17.8339 46.0723 18.2148C45.4983 18.5981 44.7484 18.7886 43.8157 18.7886C42.8477 18.7886 42.0365 18.5957 41.4697 18.1937ZM44.6353 16.2323C44.7905 15.8231 44.8705 15.1575 44.8705 14.2309V10.3292C44.8705 9.43077 44.7929 8.77225 44.6353 8.35833C44.4777 7.94206 44.2026 7.7351 43.8074 7.7351C43.4265 7.7351 43.156 7.94206 43.0008 8.35833C42.8432 8.77461 42.7656 9.43077 42.7656 10.3292V14.2309C42.7656 15.1575 42.8408 15.8254 42.9914 16.2323C43.1419 16.6415 43.4123 16.8461 43.8074 16.8461C44.2026 16.8461 44.4777 16.6415 44.6353 16.2323Z"/>
									<path fill="#FFFFFF"
												d="M56.8154 18.5634H54.6094L54.3648 17.03H54.3037C53.7039 18.1871 52.8055 18.7656 51.6061 18.7656C50.7759 18.7656 50.1621 18.4928 49.767 17.9496C49.3719 17.4039 49.1743 16.5526 49.1743 15.3955V6.03751H51.9942V15.2308C51.9942 15.7906 52.0553 16.188 52.1776 16.4256C52.2999 16.6631 52.5045 16.783 52.7914 16.783C53.036 16.783 53.2712 16.7078 53.497 16.5573C53.7228 16.4067 53.8874 16.2162 53.9979 15.9858V6.03516H56.8154V18.5634Z"/>
									<path fill="#FFFFFF"
												d="M64.4755 3.68758H61.6768V18.5629H58.9181V3.68758H56.1194V1.42041H64.4755V3.68758Z"/>
									<path fill="#FFFFFF"
												d="M71.2768 18.5634H69.0708L68.8262 17.03H68.7651C68.1654 18.1871 67.267 18.7656 66.0675 18.7656C65.2373 18.7656 64.6235 18.4928 64.2284 17.9496C63.8333 17.4039 63.6357 16.5526 63.6357 15.3955V6.03751H66.4556V15.2308C66.4556 15.7906 66.5167 16.188 66.639 16.4256C66.7613 16.6631 66.9659 16.783 67.2529 16.783C67.4974 16.783 67.7326 16.7078 67.9584 16.5573C68.1842 16.4067 68.3488 16.2162 68.4593 15.9858V6.03516H71.2768V18.5634Z"/>
									<path fill="#FFFFFF"
												d="M80.609 8.0387C80.4373 7.24849 80.1621 6.67699 79.7812 6.32186C79.4002 5.96674 78.8757 5.79035 78.2078 5.79035C77.6904 5.79035 77.2059 5.93616 76.7567 6.23014C76.3075 6.52412 75.9594 6.90747 75.7148 7.38489H75.6937V0.785645H72.9773V18.5608H75.3056L75.5925 17.3755H75.6537C75.8724 17.7988 76.1993 18.1304 76.6344 18.3774C77.0695 18.622 77.554 18.7443 78.0855 18.7443C79.038 18.7443 79.7412 18.3045 80.1904 17.4272C80.6396 16.5476 80.8653 15.1765 80.8653 13.3092V11.3266C80.8653 9.92722 80.7783 8.82892 80.609 8.0387ZM78.0243 13.1492C78.0243 14.0617 77.9867 14.7767 77.9114 15.2941C77.8362 15.8115 77.7115 16.1808 77.5328 16.3971C77.3564 16.6158 77.1165 16.724 76.8178 16.724C76.585 16.724 76.371 16.6699 76.1734 16.5594C75.9759 16.4512 75.816 16.2866 75.6937 16.0702V8.96062C75.7877 8.6196 75.9524 8.34209 76.1852 8.12337C76.4157 7.90465 76.6697 7.79646 76.9401 7.79646C77.2271 7.79646 77.4481 7.90935 77.6034 8.13278C77.7564 8.35855 77.8691 8.73485 77.9303 9.26636C77.9914 9.79787 78.022 10.5528 78.022 11.5335V13.1492H78.0243Z"/>
									<path fill="#FFFFFF"
												d="M84.8657 13.8712C84.8657 14.6755 84.8892 15.2776 84.9363 15.6798C84.9833 16.0819 85.0821 16.3736 85.2326 16.5594C85.3831 16.7428 85.6136 16.8345 85.9264 16.8345C86.3474 16.8345 86.639 16.6699 86.7942 16.343C86.9518 16.0161 87.0365 15.4705 87.0506 14.7085L89.4824 14.8519C89.4965 14.9601 89.5035 15.1106 89.5035 15.3011C89.5035 16.4582 89.186 17.3237 88.5534 17.8952C87.9208 18.4667 87.0247 18.7536 85.8676 18.7536C84.4777 18.7536 83.504 18.3185 82.9466 17.446C82.3869 16.5735 82.1094 15.2259 82.1094 13.4008V11.2136C82.1094 9.33452 82.3987 7.96105 82.9772 7.09558C83.5558 6.2301 84.5459 5.79736 85.9499 5.79736C86.9165 5.79736 87.6597 5.97375 88.1771 6.32888C88.6945 6.684 89.059 7.23433 89.2707 7.98457C89.4824 8.7348 89.5882 9.76961 89.5882 11.0913V13.2362H84.8657V13.8712ZM85.2232 7.96811C85.0797 8.14449 84.9857 8.43377 84.9363 8.83593C84.8892 9.2381 84.8657 9.84722 84.8657 10.6657V11.5641H86.9283V10.6657C86.9283 9.86133 86.9001 9.25221 86.846 8.83593C86.7919 8.41966 86.6931 8.12803 86.5496 7.95635C86.4062 7.78702 86.1851 7.7 85.8864 7.7C85.5854 7.70235 85.3643 7.79172 85.2232 7.96811Z"/>
								</g>
							</svg>
						</div>
					</div>
					<div className="header-center">
						<div className="search-container">
							<input type="text" className="search-input" placeholder="Rechercher" value={searchInput} onChange={(e) => setSearchInput(e.target.value)} onKeyPress={handleKeyPress}/>
							<button className="search-button" onClick={handleSearch} disabled={loading}>
								<svg viewBox="0 0 24 24" width="24" height="24">
									<path fill="#fff"
												d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
								</svg>
							</button>
						</div>
					</div>
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
				<div className="layout-container">
					<div className="sidebar-wrapper">
						<aside className="youtube-sidebar">
							<div className="sidebar-section">
								<a href="#" className="sidebar-item active">
									<span className="icon">
										<svg viewBox="0 0 24 24">
											<path
												fill="#fff"
												d="m11.485 2.143-8 4.8-2 1.2a1 1 0 001.03 1.714L3 9.567V20a2 2 0 002 2h5v-8h4v8h5a2 2 0 002-2V9.567l.485.29a1 1 0 001.03-1.714l-2-1.2-8-4.8a1 1 0 00-1.03 0Z"></path>
										</svg>
									</span>
									<span>Accueil</span>
								</a>
								<a href="#" className="sidebar-item">
									<span className="icon">📹</span>
									<span>Shorts</span>
								</a>
								<a href="#" className="sidebar-item">
									<span className="icon">📺</span>
									<span>Abonnements</span>
								</a>
							</div>

							<div className="sidebar-section">
								<div className="sidebar-title">Vous</div>
								<a href="#" className="sidebar-item">
									<span className="icon">🕐</span>
									<span>Historique</span>
								</a>
								<a href="#" className="sidebar-item">
									<span className="icon">📋</span>
									<span>Playlists</span>
								</a>
								<a href="#" className="sidebar-item">
									<span className="icon">▶️</span>
									<span>Vos vidéos</span>
								</a>
								<a href="#" className="sidebar-item">
									<span className="icon">⏱️</span>
									<span>À regarder plus tard</span>
								</a>
								<a href="#" className="sidebar-item">
									<span className="icon">👍</span>
									<span>Vidéos j'aime</span>
								</a>
								<a href="#" className="sidebar-item">
									<span className="icon">⬇️</span>
									<span>Téléchargements</span>
								</a>
								<a href="#" className="sidebar-item">
									<span className="icon">✂️</span>
									<span>Vos clips</span>
								</a>
							</div>

							<div className="sidebar-section">
								<div className="sidebar-title">Abonnements</div>
								<a href="#" className="sidebar-item">
									<span className="icon">▶️</span>
									<span>YouTube</span>
								</a>
								<a href="#" className="sidebar-item">
									<span className="icon">🏍️</span>
									<span>100% MOTOS</span>
								</a>
								<a href="#" className="sidebar-item">
									<span className="icon">🎤</span>
									<span>Aaed Musa</span>
								</a>
								<a href="#" className="sidebar-item">
									<span className="icon">🎯</span>
									<span>Abrège</span>
								</a>
							</div>

							<div className="sidebar-section">
								<div className="sidebar-title">Explorer</div>
								<a href="#" className="sidebar-item">
									<span className="icon">🎵</span>
									<span>Musique</span>
								</a>
							</div>
						</aside>
					</div>
					<div className="content-wrapper">
						{loading && (
							<p style={{textAlign: 'center', padding: '1rem', color: '#f1f1f1'}}>
								Chargement des vidéos...
							</p>
						)}
						{error && (
							<p style={{color: 'red', textAlign: 'center', padding: '1rem'}}>
								{error}
							</p>
						)}
						{videos && videos.length > 0 && <VideoGrid videos={videos}/>}
					</div>
				</div>
			</div>
		</ErrorBoundary>
	);
}

function App() {
	return (
		<Router>
			<Routes>
				<Route path="/" element={<MainApp/>}/>
			</Routes>
		</Router>
	);
}

export default App;