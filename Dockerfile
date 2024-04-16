FROM node:20-bookworm

USER node
WORKDIR /home/node

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/home/node/.cargo/bin:${PATH}"

RUN mkdir -p /home/node/.npm-global
RUN npm config set prefix /home/node/.npm-global
RUN echo "PATH=\"/home/node/.npm-global/bin:/home/node/.cargo/bin:\$PATH\"" >> /home/node/.bashrc

RUN sh -c "$(curl -sSfL https://release.solana.com/v1.18.11/install)" \
    && cargo install anchor-cli
RUN echo "PATH=\"/home/node/.local/share/solana/install/active_release/bin:\$PATH\"" >> /home/node/.bashrc

RUN npm install -g typescript

# To ensure that the `/home/node/.config/solana` volume is going to be owned by
# the `node` user. Otherwise it'd be owned by root.
RUN mkdir -p /home/node/.config/solana

ENTRYPOINT []
