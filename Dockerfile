FROM node:18-alpine

# Créer un utilisateur non-root pour correspondre au backend
ARG USER_ID=1000
ARG GROUP_ID=1000
RUN addgroup -g ${GROUP_ID} mygroup && \
    adduser -u ${USER_ID} -G mygroup -D myuser

# Définir le répertoire de travail
WORKDIR /usr/src/myapp

# Copier package.json et installer les dépendances
COPY package*.json ./
RUN npm install

# Passer à l'utilisateur non-root
USER myuser

# Exposer le port pour React
EXPOSE 3000

# Commande par défaut : shell interactif
CMD ["bash"]