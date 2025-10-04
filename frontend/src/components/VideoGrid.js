import React, { useState } from 'react';
import './VideoGrid.css';

const VideoGrid = ({ videos }) => {
	const [isModalOpen, setIsModalOpen] = useState(false);
	const [selectedvideo_id, setSelectedvideo_id] = useState(null);

	if (!videos || videos.length === 0) {
		return <p style={{ textAlign: 'center', padding: '2rem' }}>Aucune vidéo à afficher</p>;
	}

	const openModal = (video_id) => {
		setSelectedvideo_id(video_id);
		setIsModalOpen(true);
	};

	const closeModal = () => {
		setIsModalOpen(false);
		setSelectedvideo_id(null);
	};

	return (
		<div>
			<div className="video-grid">
				{videos.map((video, index) => (
					<div key={index} className="video-card" onClick={() => openModal(video.video_id)}>
						<img
							src={video.thumbnail}
							alt={video.title}
							className="video-thumbnail"
							style={{ cursor: 'pointer' }}
						/>
						<div className="video-info">
							<h3 className="video-title">{video.title}</h3>
							<p className="video-channel">{video.channel_title}</p>
							<p className="video-date">
								{new Date(video.published_at).toLocaleDateString('fr-FR', {
									year: 'numeric',
									month: 'long',
									day: 'numeric'
								})}
							</p>
						</div>
					</div>
				))}
			</div>

			{isModalOpen && (
				<div className="modal-overlay" onClick={closeModal}>
					<div className="modal-content" onClick={(e) => e.stopPropagation()}>
						<button className="close-button" onClick={closeModal}>X</button>
						<iframe
							width="800"
							height="450"
							src={`https://www.youtube.com/embed/${selectedvideo_id}?autoplay=1`}
							title="YouTube video player"
							frameBorder="0"
							allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
							allowFullScreen
						></iframe>
					</div>
				</div>
			)}
		</div>
	);
};

export default VideoGrid;
