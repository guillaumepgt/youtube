FROM rust:latest

ARG USER_ID=1000
ARG GROUP_ID=1000

# Check if group and user exist, create if not
RUN if getent group ${GROUP_ID} >/dev/null; then \
        echo "Group ID ${GROUP_ID} already exists"; \
    else \
        groupadd -g ${GROUP_ID} mygroup; \
    fi && \
    if getent passwd ${USER_ID} >/dev/null; then \
        echo "User ID ${USER_ID} already exists"; \
    else \
        useradd -u ${USER_ID} -g ${GROUP_ID} -m myuser; \
    fi

WORKDIR /usr/src/myapp
COPY . .

USER myuser
EXPOSE 8080
WORKDIR /usr/src/myapp

CMD ["cargo", "run", "--release"]