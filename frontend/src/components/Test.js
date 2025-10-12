import React, { useState } from 'react';
import './Test.css';

export default function Test() {
	const [currentFilter, setCurrentFilter] = useState('all');
	const [subscribedChannels, setSubscribedChannels] = useState(new Set([1, 3, 5, 7]));

	const channels = [
		{ id: 1, name: 'TechChannel', subscribers: '2.5M', icon: '💻', header: '' },
		{ id: 2, name: 'Gaming Pro', subscribers: '1.8M', icon: '🎮', header: 'header2' },
		{ id: 3, name: 'Music Daily', subscribers: '5.2M', icon: '🎵', header: 'header3' },
		{ id: 4, name: 'Fitness Guru', subscribers: '890K', icon: '💪', header: 'header4' },
		{ id: 5, name: 'Cooking Secrets', subscribers: '3.1M', icon: '👨‍🍳', header: '' },
		{ id: 6, name: 'Travel Vlog', subscribers: '1.5M', icon: '✈️', header: 'header2' },
		{ id: 7, name: 'Dev Tutorials', subscribers: '456K', icon: '👨‍💻', header: 'header3' },
		{ id: 8, name: 'Movie Reviews', subscribers: '2.2M', icon: '🎬', header: 'header4' },
	];

	const toggleSubscribe = (channelId) => {
		const newSubscribed = new Set(subscribedChannels);
		if (newSubscribed.has(channelId)) {
			newSubscribed.delete(channelId);
		} else {
			newSubscribed.add(channelId);
		}
		setSubscribedChannels(newSubscribed);
	};

	const getFilteredChannels = () => {
		if (currentFilter === 'popular') return channels.slice(0, 4);
		if (currentFilter === 'new') return channels.slice(4);
		return channels;
	};

	return (
		<div className="subscriptions-container">
			<div className="subscriptions-layout">
				<main className="subscriptions-main">
					<div className="subscriptions-grid">
						{getFilteredChannels().map((channel) => {
							const isSubscribed = subscribedChannels.has(channel.id);
							return (
								<div key={channel.id} className="subscription-card">
									<div className={`channel-header ${channel.header}`}>
										<div className="channel-avatar">{channel.icon}</div>
									</div>
									<div className="card-content">
										<div className="channel-name">{channel.name}</div>
										<div className="subscriber-count">{channel.subscribers} abonnés</div>
									</div>
								</div>
							);
						})}
					</div>
				</main>
			</div>
		</div>
	);
}