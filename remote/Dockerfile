# Stage 1: Build the application
FROM node:20 as builder
WORKDIR /remote
COPY package*.json ./
RUN npm install
COPY . .
RUN npm run build
ENV CORS=false \
    API_BASE_URL=https://tank-game.flmichel.duckdns.org/api

# Stage 2: Create the production image with Nginx
FROM nginx:latest
COPY --from=builder /remote/public /usr/share/nginx/html
EXPOSE 80