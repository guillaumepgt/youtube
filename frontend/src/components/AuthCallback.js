// Créez ce composant : src/components/AuthCallback.jsx

import { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';

function AuthCallback() {
	const [searchParams] = useSearchParams();
	const navigate = useNavigate();
	const [error, setError] = useState(null);
	const [loading, setLoading] = useState(true);

	useEffect(() => {
		const code = searchParams.get('code');
		const errorParam = searchParams.get('error');

		if (errorParam) {
			setError('Authentification annulée ou échouée');
			setLoading(false);
			setTimeout(() => navigate('/'), 3000);
			return;
		}

		if (!code) {
			setError('Aucun code d\'authentification reçu');
			setLoading(false);
			setTimeout(() => navigate('/'), 3000);
			return;
		}

		// Échanger le code contre des tokens
		fetch('/api/exchange-code', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
			},
			body: JSON.stringify({ code }),
		})
			.then(response => {
				if (!response.ok) {
					throw new Error('Erreur lors de l\'échange du code');
				}
				return response.json();
			})
			.then(data => {
				console.log('Tokens reçus:', data);

				// Sauvegarder les tokens (localStorage, context, Redux, etc.)
				localStorage.setItem('access_token', data.access_token);
				localStorage.setItem('refresh_token', data.refresh_token);
				localStorage.setItem('expires_in', data.expires_in);
				localStorage.setItem('token_timestamp', Date.now());

				// Rediriger vers la page principale
				navigate('/', { replace: true });
			})
			.catch(err => {
				console.error('Erreur:', err);
				setError('Erreur lors de l\'authentification');
				setLoading(false);
				setTimeout(() => navigate('/'), 3000);
			});
	}, [searchParams, navigate]);

	if (loading) {
		return (
			<div style={{
				display: 'flex',
				flexDirection: 'column',
				alignItems: 'center',
				justifyContent: 'center',
				height: '100vh',
				fontFamily: 'Arial, sans-serif'
			}}>
				<div style={{
					border: '4px solid #f3f3f3',
					borderTop: '4px solid #3498db',
					borderRadius: '50%',
					width: '40px',
					height: '40px',
					animation: 'spin 1s linear infinite',
				}}></div>
				<p style={{ marginTop: '20px', fontSize: '18px', color: '#555' }}>
					Authentification en cours...
				</p>
				<style>{`
          @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
          }
        `}</style>
			</div>
		);
	}

	if (error) {
		return (
			<div style={{
				display: 'flex',
				flexDirection: 'column',
				alignItems: 'center',
				justifyContent: 'center',
				height: '100vh',
				fontFamily: 'Arial, sans-serif'
			}}>
				<p style={{ fontSize: '20px', color: '#e74c3c' }}>❌ {error}</p>
				<p style={{ marginTop: '10px', color: '#555' }}>
					Redirection vers la page d'accueil...
				</p>
			</div>
		);
	}

	return null;
}

export default AuthCallback;